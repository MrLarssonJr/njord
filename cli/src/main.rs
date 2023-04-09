mod nordigen;

use std::error::Error;
use std::fmt::{Display, Formatter};
use chrono::{DateTime, Utc};
use dialoguer::{Confirm, FuzzySelect, Input};
use dialoguer::theme::{ColorfulTheme, Theme};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use crate::nordigen::get_available_institutions;

static APP_NAME: &'static str = "njord";

fn main() -> Result<(), Box<dyn Error>> {
	let client = Client::new();
	let theme = ColorfulTheme::default();
	let mut state: State = confy::load(APP_NAME, None)?;

	while let Some(new_state) = state.prompt(&client, &theme)? {
		state = new_state;
		confy::store(APP_NAME, None, &state)?;
	}

	Ok(())
}

#[derive(Serialize, Deserialize, Default, Debug)]
enum State {
	#[default]
	Blank,
	GotClientCredentials {
		client_credentials: ClientCredentials,
	},
	GotAccessToken {
		client_credentials: ClientCredentials,
		token: Token,
	},
	InstitutionsChosen {
		client_credentials: ClientCredentials,
		token: Token,
		institutions: Vec<Institution>
	}
}

impl State {
	fn prompt<T: Theme>(self, client: &Client, theme: &T) -> Result<Option<State>, Box<dyn Error>> {
		match self {
			State::Blank => {
				let id: String = Input::with_theme(theme)
					.with_prompt("Client ID")
					.interact_text()?;

				let secret = Input::with_theme(theme)
					.with_prompt("Client secret")
					.interact_text()?;

				let client_credentials = ClientCredentials { id, secret };

				Ok(Some(State::GotClientCredentials { client_credentials }))
			}
			State::GotClientCredentials { client_credentials } => {
				let acquire = Confirm::with_theme(theme)
					.with_prompt("Missing token, acquire?")
					.interact()?;

				if !acquire {
					return Ok(None);
				}

				let token = nordigen::new_token(client, &client_credentials)?;

				Ok(Some(State::GotAccessToken {
					client_credentials,
					token
				}))
			}
			State::GotAccessToken { client_credentials, token } => {
				let mut available_institutions = get_available_institutions(client, &token)?;
				let mut chosen_institutions = Vec::new();

				loop {
					let chosen_institution = FuzzySelect::with_theme(theme)
						.with_prompt("Please choose the institutions you have accounts with")
						.items(&available_institutions)
						.interact()?;

					chosen_institutions.push(available_institutions.remove(chosen_institution));

					println!("You've selected the following institutions:");
					for institution in chosen_institutions.iter() {
						println!(" - {institution}");
					}

					let again = Confirm::with_theme(theme)
						.with_prompt("Select additional institutions?")
						.interact()?;

					if !again { break; }
				}

				Ok(Some(State::InstitutionsChosen {
					client_credentials,
					token,
					institutions: chosen_institutions
				}))
			}
			state @ State::InstitutionsChosen { .. } => {
				println!("{state:?}");
				Ok(None)
			}
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientCredentials {
	id: String,
	secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Token {
	access: TokenPart,
	refresh: TokenPart,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenPart {
	secret: String,
	expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Institution {
	id: String,
	name: String,
	countries: Vec<String>,
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
