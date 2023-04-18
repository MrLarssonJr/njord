use std::fmt::{Display, Formatter};
use std::rc::Rc;
use chrono::{Duration, NaiveDate};
use color_eyre::eyre;
use inquire::Select;
use rust_decimal::Decimal;
use rust_decimal::prelude::{Zero};
use crate::nordigen::account::Account;
use crate::nordigen::transaction::RawTransaction;

#[derive(Debug)]
pub enum Transaction {
	Normal(NormalTransaction),
	Transfer(TransferTransaction)
}

impl Display for Transaction {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Transaction::Normal(t) => write!(f, "{t}"),
			Transaction::Transfer(t) => write!(f, "{t}"),
		}
	}
}

#[derive(Debug)]
pub struct NormalTransaction {
	account: Rc<Account>,
	amount: Decimal,
	currency: String,
	date: NaiveDate,
	additional_info: String,
}

impl Display for NormalTransaction {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{} ", self.date)?;
		if self.amount < Decimal::zero() {
			write!(f, "from: {} ", self.account)?;
			write!(f, "{} {} ", -self.amount, self.currency)?;
		} else {
			write!(f, "to: {} ", self.account)?;
			write!(f, "{} {} ", self.amount, self.currency)?;
		}
		write!(f, "{}", self.additional_info)
	}
}

#[derive(Debug)]
pub struct TransferTransaction {
	from: Rc<Account>,
	to: Rc<Account>,
	amount: Decimal,
	currency: String,
	date: NaiveDate,
	from_additional_info: String,
	to_additional_info: String,
}

impl Display for TransferTransaction {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{} ", self.date)?;
		if self.amount < Decimal::zero() {
			write!(f, "from: {} ", self.from)?;
			write!(f, "to: {} ", self.to)?;
			write!(f, "{} {} ", -self.amount, self.currency)?;
		} else {
			write!(f, "from: {} ", self.to)?;
			write!(f, "to: {} ", self.from)?;
			write!(f, "{} {} ", self.amount, self.currency)?;
		}
		write!(f, "{} ", self.from_additional_info)?;
		write!(f, "{}", self.to_additional_info)
	}
}

impl From<&(RawTransaction, Rc<Account>)> for Transaction {
	fn from((raw_transaction, account): &(RawTransaction, Rc<Account>)) -> Self {
		Transaction::Normal(NormalTransaction {
			account: account.clone(),
			amount: raw_transaction.amount,
			currency: raw_transaction.currency.clone(),
			date: raw_transaction.date,
			additional_info: raw_transaction.additional_info.clone(),
		})
	}
}

pub fn match_transactions(raw_transactions: &[(RawTransaction, Rc<Account>)]) -> eyre::Result<Vec<Transaction>> {
	let mut transactions: Vec<_> = raw_transactions.into_iter()
		.map(Transaction::from)
		.collect();

	let mut index = 0;

	while index < transactions.len() {
		let (_, unmatched_transactions) = transactions.split_at(index);
		let Some((target, candidates)) = unmatched_transactions.split_first() else { index += 1; continue };
		let Transaction::Normal(target) = target else { index += 1; continue };
		let squared_errors = find_matches(candidates, target);

		let Some((picked_transaction, picked_index)) = (match pick_match(&squared_errors) {
			Match::HumanInterventionRequired(close_candidates) => ask_human(target, &close_candidates)?,
			Match::ObviousChoice(transaction, index) => Some((transaction, index)),
			Match::NoMatch => None,
		}) else { index += 1; continue };

		let (from, to) = if target.amount < Decimal::zero() {
			(target, picked_transaction)
		} else {
			(picked_transaction, target)
		};

		let transfer = TransferTransaction {
			from: from.account.clone(),
			to: to.account.clone(),
			amount: from.amount.abs(),
			currency: from.currency.clone(),
			date: from.date,
			from_additional_info: from.additional_info.clone(),
			to_additional_info: to.additional_info.clone(),
		};

		*(&mut transactions[index]) = Transaction::Transfer(transfer);
		transactions.remove(index + 1 + picked_index);

		index += 1;
	}

	Ok(transactions)
}

fn ask_human<'a>(target: &NormalTransaction, scored_candidates: &[(&'a NormalTransaction, usize, Duration)]) -> eyre::Result<Option<(&'a NormalTransaction, usize)>> {
	println!("Trying to find figure out if the following transaction is part of a transfer");
	println!("{target}");

	let options = scored_candidates.into_iter()
		.map(|(transaction, index, error)| Candidate {
			index: *index,
			error: *error,
			transaction,
		})
		.collect::<Vec<_>>();

	Ok(Select::new("Which is other half of transfer?", options).prompt_skippable()?
		.map(|candidate| (candidate.transaction, candidate.index)))
}

struct Candidate<'a> {
	index: usize,
	error: Duration,
	transaction: &'a NormalTransaction
}

impl<'a> Display for Candidate<'a> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "[{}] ", self.error)?;
		write!(f, "{}", self.transaction)
	}
}

enum Match<'a> {
	HumanInterventionRequired(Vec<(&'a NormalTransaction, usize, Duration)>),
	ObviousChoice(&'a NormalTransaction, usize),
	NoMatch,
}

fn pick_match<'a>(scored_candidates: &[(&'a NormalTransaction, usize, Duration)]) -> Match<'a> {
	let perfect_candidates = scored_candidates.into_iter()
		.copied()
		.filter(|(_, _ , error)| *error == Duration::zero())
		.collect::<Vec<_>>();

	if let Some((candidate, index, _)) = perfect_candidates.first() {
		return if perfect_candidates.len() == 1 {
			Match::ObviousChoice(*candidate, *index)
		} else {
			Match::HumanInterventionRequired(perfect_candidates)
		}
	}

	let close_candidates = scored_candidates.iter()
		.copied()
		.filter(|(_, _, error)| Duration::days(-5) < *error && *error < Duration::days(5))
		.collect::<Vec<_>>();

	return if close_candidates.is_empty() {
		Match::NoMatch
	} else {
		Match::HumanInterventionRequired(close_candidates)
	};
}

fn find_matches<'a>(candidates: &'a [Transaction], target: &NormalTransaction) -> Vec<(&'a NormalTransaction, usize, Duration)> {
	let mut res = vec![];

	for (index, candidate) in candidates.iter().enumerate() {
		let Transaction::Normal(candidate) = candidate else { continue };
		let Some(squared_error) = evaluate_match(target, candidate) else { continue };
		res.push((candidate, index, squared_error));
	}

	res.sort_unstable_by_key( |scored_candidate| scored_candidate.2);

	res
}

fn evaluate_match(target: &NormalTransaction, candidate: &NormalTransaction) -> Option<Duration> {
	if target.account.id == candidate.account.id { return None };
	if target.currency != candidate.currency { return None };
	let amount_sum = target.amount + candidate.amount;
	if amount_sum != Decimal::zero() { return None; }

	let date_diff = target.date - candidate.date;

	Some(date_diff)
}
