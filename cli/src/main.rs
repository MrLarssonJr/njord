mod nordigen;
mod config;
mod client_credentials;
mod token;
mod institution;
mod interactions;
mod requisition;

use color_eyre::eyre;
use color_eyre::eyre::eyre;
use once_cell::sync::Lazy;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use crate::config::Config;
use crate::institution::Institution;
use crate::token::Token;

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
	let loaded_config = confy::load::<Config>(APP_NAME, Some("config"))?;

	let client_credentials = if let Some(client_credentials) =  loaded_config.client_credentials {
		client_credentials
	} else {
		interactions::ClientCredentialsInput::prompt()?
	};

	let mut token: Token = Token::new(&client_credentials)?;

	let reuse_selected_institutions = interactions::ReuseConfirm::new(&loaded_config.selected_institutions).prompt()?;
	let mut selected_institutions = if reuse_selected_institutions {
		loaded_config.selected_institutions
	} else {
		let available_institutions = Institution::list(&client_credentials, &mut token)?;

		 interactions::InstitutionSelect::new(available_institutions)
			.prompt()?
	};

	let mut requisitions = selected_institutions.iter_mut()
		.map(|si| si.get_requisition(&client_credentials, &mut token))
		.collect::<Result<Vec<_>, _ >>()?;

	for (index, requisition) in requisitions.iter_mut().enumerate() {
		if requisition.is_linked() {
			continue;
		}

		requisition.open_link()?;
		interactions::AcceptedConfirm::new(&selected_institutions[index]).prompt()?;
		requisition.update(&client_credentials, &mut token)?;

		if !requisition.is_linked() {
			Err(eyre!("Account still unlinked after returning!"))?;
		}
	}

	let account_ids = requisitions.iter()
		.flat_map(|req| req.accounts.iter().map(String::as_str))
		.collect::<Vec<_>>();

	println!("{account_ids:#?}");

	let save_config = Config {
		client_credentials: Some(client_credentials),
		token: Some(token),
		selected_institutions,
	};

	confy::store(APP_NAME, Some("config"), save_config)?;

	Ok(())
}
