use chrono::{DateTime, Local};
use color_eyre::eyre;
use serde::{Deserialize, Serialize};
use crate::nordigen::client_credentials::ClientCredentials;
use crate::HTTP_CLIENT;
use crate::nordigen::account::Account;
use crate::nordigen::http_interface;
use crate::nordigen::token::Token;

#[derive(Debug, Serialize, Deserialize)]
pub struct Requisition {
	pub id: String,
	pub created: DateTime<Local>,
	pub status: String,
	pub accounts: Vec<Account>,
	pub link: String,
}

impl Requisition {
	pub fn new(client_credentials: &ClientCredentials, token: &mut Token, institution_id: &str) -> eyre::Result<Requisition> {
		let access_token = token.get_access_token(client_credentials)?;

		let body = http_interface::requisitions::PostRequestBody {
			redirect: "https://njord.jesperlarsson.me/requisition_return",
			institution_id,
		};

		let res = http_interface::requisitions::post(&HTTP_CLIENT, access_token, &body)?;

		let accounts = res.accounts.iter()
			.map(|account_id| { Account::get(client_credentials, token, account_id) })
			.flat_map(|res| match res {
				Ok(account) if account.is_available() => Some(Ok(account)),
				Ok(_) => None,
				Err(e) => Some(Err(e))
			})
			.collect::<Result<Vec<_>, _>>()?;


		Ok(Requisition {
			id: res.id,
			created: res.created,
			status: res.status,
			accounts,
			link: res.link,
		})
	}

	pub fn get(client_credentials: &ClientCredentials, token: &mut Token, id: &str) -> eyre::Result<Requisition> {
		let access_token = token.get_access_token(client_credentials)?;

		let res = http_interface::requisitions::get(&HTTP_CLIENT, access_token, id)?;

		let accounts = res.accounts.iter()
			.map(|account_id| { Account::get(client_credentials, token, account_id) })
			.flat_map(|res| match res {
				Ok(account) if account.is_available() => Some(Ok(account)),
				Ok(_) => None,
				Err(e) => Some(Err(e))
			})
			.collect::<Result<Vec<_>, _>>()?;

		Ok(Requisition {
			id: res.id,
			created: res.created,
			status: res.status,
			accounts,
			link: res.link,
		})
	}

	pub fn update(&mut self, client_credentials: &ClientCredentials, token: &mut Token) -> eyre::Result<()> {
		*self = Requisition::get(client_credentials, token, &self.id)?;

		Ok(())
	}

	pub fn is_linked(&self) -> bool {
		self.status == "LN"
	}

	pub fn open_link(&self) -> eyre::Result<()> {
		Ok(open::that(&self.link)?)
	}
}
