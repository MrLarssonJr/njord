use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use color_eyre::eyre;
use crate::client_credentials::ClientCredentials;
use crate::HTTP_CLIENT;
use crate::nordigen::http_interface;
use crate::requisition::Requisition;
use crate::token::Token;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Institution {
	pub id: String,
	pub name: String,
	pub countries: Vec<String>,
	pub requisition_id: Option<String>,
}

impl Institution {
	pub fn list(client_credentials: &ClientCredentials, token: &mut Token) -> eyre::Result<Vec<Institution>> {
		let access_token = token.get_access_token(client_credentials)?;

		let response = http_interface::institutions::list(&HTTP_CLIENT, access_token)?;
		let response = response.into_iter()
			.map(|res| Institution {
				id: res.id,
				name: res.name,
				countries: res.countries,
				requisition_id: None,
			})
			.collect();

		Ok(response)
	}

	pub fn get_requisition(&mut self, client_credentials: &ClientCredentials, token: &mut Token) -> eyre::Result<Requisition> {
		let req = if let Some(requisition_id) = &self.requisition_id {
			Requisition::get(client_credentials, token, requisition_id)
				.or_else(|_| Requisition::new(client_credentials, token, &self.id))?
		} else {
			Requisition::new(client_credentials, token, &self.id)?
		};

		self.requisition_id = Some(req.id.clone());

		Ok(req)
	}
}

impl Display for Institution {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let name = &self.name;

		write!(f, "{name} [")?;
		let mut iter = self.countries.iter();
		if let Some(country) = iter.next() {
			write!(f, "{country}")?;
		}
		for country in iter {
			write!(f, ", {country}")?;
		}
		write!(f, "]")
	}
}
