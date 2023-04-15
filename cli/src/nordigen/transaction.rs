use chrono::NaiveDate;
use color_eyre::eyre;
use rust_decimal::Decimal;
use crate::nordigen::client_credentials::ClientCredentials;
use crate::HTTP_CLIENT;
use crate::nordigen::http_interface;
use crate::nordigen::token::Token;

#[derive(Debug)]
pub struct Transaction {
	pub account: String,
	pub date: NaiveDate,
	pub currency: String,
	pub amount: Decimal,
	pub additional_info: String,
	pub id: String,
}

impl Transaction {
	pub fn list_in_account(client_credentials: &ClientCredentials, token: &mut Token, account_id: &str) -> eyre::Result<Vec<Transaction>> {
		let token = token.get_access_token(&client_credentials)?;

		let res = http_interface::accounts::transactions::get(&HTTP_CLIENT, token, account_id)?;
		let booked_transactions = res.transactions.booked;

		let transactions = booked_transactions.into_iter()
			.map(|booked_transaction| Transaction {
				account: account_id.to_string(),
				date: booked_transaction.value_date,
				currency: booked_transaction.transaction_amount.currency,
				amount: booked_transaction.transaction_amount.amount,
				additional_info: booked_transaction.additional_information,
				id: booked_transaction.transaction_id,
			})
			.collect();

		Ok(transactions)
	}
}
