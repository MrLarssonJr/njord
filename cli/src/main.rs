mod nordigen;
mod interactions;
mod matcher;

use std::io::stdout;
use std::ops::Deref;
use chrono::NaiveDate;
use color_eyre::eyre;
use once_cell::sync::Lazy;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::matcher::{match_transactions, Transaction};
use crate::nordigen::account::Account;
use crate::nordigen::get_raw_transactions;

pub static APP_NAME: &'static str = "njord";
pub static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
	let mut default_headers = HeaderMap::new();
	default_headers.insert("Accept", HeaderValue::from_static("application/json"));

	Client::builder()
		.default_headers(default_headers)
		.build()
		.expect("unable to build HTTP client")
});

fn main() -> eyre::Result<()> {
	let raw_transactions = get_raw_transactions()?;
	let matched_transactions = match_transactions(&raw_transactions)?;


	let mut writer = csv::WriterBuilder::new().from_writer(stdout());
	for matched_transaction in matched_transactions {
		let record = OutputFormat::from(matched_transaction);
		writer.serialize(record)?;
	}

	Ok(())
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
struct OutputFormat {
	date: NaiveDate,
	account_from: String,
	account_to: Option<String>,
	amount: Decimal,
	currency: String,
	description: String,
}

impl From<Transaction> for OutputFormat {
	fn from(transaction: Transaction) -> Self {
		match transaction {
			Transaction::Normal(transaction) => OutputFormat {
				date: transaction.date,
				account_from: {
					let Account { bban, iban, name, display_name, .. } = transaction.account.deref().clone();
					name.or(display_name).or(bban).or(iban).unwrap_or("unknown".into())
				},
				account_to: None,
				amount: transaction.amount,
				currency: transaction.currency,
				description: transaction.additional_info.unwrap_or_default(),
			},
			Transaction::Transfer(transaction) => OutputFormat {
				date: transaction.date,
				account_from: {
					let Account { bban, iban, name, display_name, .. } = transaction.from.deref().clone();
					name.or(display_name).or(bban).or(iban).unwrap_or("unknown".into())
				},
				account_to: {
					let Account { bban, iban, name, display_name, .. } = transaction.to.deref().clone();
					Some(name.or(display_name).or(bban).or(iban).unwrap_or("unknown".into()))
				},
				amount: transaction.amount,
				currency: transaction.currency,
				description: format!("from: {} to: {}", transaction.from_additional_info.unwrap_or_default(), transaction.to_additional_info.unwrap_or_default()),
			},
		}
	}
}
