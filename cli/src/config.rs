use std::error::Error;
use std::fmt::{Display, Formatter};
use chrono::{DateTime, Local};
use inquire::{Confirm, MultiSelect, Text};
use serde::{Deserialize, Serialize};
use crate::nordigen::NordigenClientContext;
use crate::{APP_NAME, HTTP_CLIENT};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
	pub client_credentials: ClientCredentials,
	pub token: Option<Token>,
	pub institutions: Vec<Institution>,
}

impl Config {
	pub fn new() -> Result<Config, Box<dyn Error>> {
		println!("Welcome to the njord configuration creation wizard!");

		let id = Text::new("Client ID").prompt()?;
		let secret = Text::new("Client secret").prompt()?;

		let client_credentials = ClientCredentials { id, secret };

		let mut config = Config {
			client_credentials,
			token: None,
			institutions: vec![],
		};

		let choose_institutions_now = Confirm::new("Select the institutions you'd like to pull data from now?")
			.with_default(true)
			.with_help_message("This can also be done later in settings")
			.prompt()?;

		if choose_institutions_now {
			config.select_institutions()?;
		}

		config.save()?;

		Ok(config)
	}

	pub fn set_client_credentials(&mut self) -> Result<&mut Self, Box<dyn Error>> {
		if let Some(id) = Text::new("Client ID").prompt_skippable()? {
			self.client_credentials.id = id;
		}
		if let Some(secret) = Text::new("Client secret").prompt_skippable()? {
			self.client_credentials.secret = secret;
		}
		Ok(self)
	}

	pub fn select_institutions(&mut self) -> Result<&mut Self, Box<dyn Error>> {
		let available_institutions = {
			let mut nordigen_client_context = NordigenClientContext::new(self, &HTTP_CLIENT)?;
			nordigen_client_context.available_institutions()?
		};

		let selected_indices = available_institutions.iter().enumerate()
			.filter(|(_, available_institution)| self.institutions.iter().any(|institution| institution.id == available_institution.id))
			.map(|(i, _)| i)
			.collect::<Vec<_>>();

		let chosen_institutions = MultiSelect::new("Choose institutions you have accounts with", available_institutions)
			.with_keep_filter(false)
			.with_default(&selected_indices)
			.prompt_skippable()?;

		if let Some(chosen_institutions) = chosen_institutions {
			self.institutions = chosen_institutions;
		}

		Ok(self)
	}

	pub fn save(&self) -> Result<(), Box<dyn Error>> {
		confy::store(APP_NAME, Some("config"), Some(&self))
			.map_err(|e| e.into())
	}
}


#[derive(Debug, Serialize, Deserialize)]
pub struct ClientCredentials {
	pub id: String,
	pub secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Token {
	pub access: TokenPart,
	pub refresh: TokenPart,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct TokenPart {
	pub secret: String,
	pub expires_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Institution {
	pub id: String,
	pub name: String,
	pub countries: Vec<String>,
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
