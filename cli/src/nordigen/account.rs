use std::fmt::{Display, Formatter};
use color_eyre::eyre;
use serde::{Deserialize, Serialize};
use crate::HTTP_CLIENT;
use crate::nordigen::client_credentials::ClientCredentials;
use crate::nordigen::http_interface;
use crate::nordigen::token::Token;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
	pub id: String,
	pub bban: Option<String>,
	pub iban: String,
	pub status: String,
	pub name: Option<String>,
	pub display_name: Option<String>
}

impl Account {
	pub fn get(client_credentials: &ClientCredentials, token: &mut Token, id: &str) -> eyre::Result<Account> {
		let access_token = token.get_access_token(client_credentials)?;

		let res = http_interface::accounts::details::get(&HTTP_CLIENT, access_token, id)?;

		Ok(Account {
			id: id.to_string(),
			bban: res.account.bban,
			iban: res.account.iban,
			status: res.account.status,
			name: res.account.name,
			display_name: res.account.display_name,
		})
	}

	pub fn is_available(&self) -> bool {
		self.status == "enabled"
	}
}

impl Display for Account {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		if let Some(display_name) = &self.display_name {
			write!(f, "{display_name}")
		} else if let Some(name) = &self.name {
			write!(f, "{name}")
		} else if let Some(bban) = &self.bban {
			write!(f, "{bban}")
		} else {
			write!(f, "{}", &self.id)
		}
	}
}
