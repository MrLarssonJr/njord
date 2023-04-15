mod nordigen;
mod interactions;

use color_eyre::eyre;
use color_eyre::eyre::eyre;
use once_cell::sync::Lazy;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use crate::nordigen::config::Config;
use crate::nordigen::institution::Institution;
use crate::nordigen::token::Token;
use crate::nordigen::transaction::Transaction;

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

	let mut transactions: Vec<Transaction> = vec![];

	for (institution_index, requisition) in requisitions.iter().enumerate() {
		let institution = &mut selected_institutions[institution_index];
		for account_id in requisition.accounts.iter() {
			let account_transactions = match Transaction::list_in_account(&client_credentials, &mut token, account_id) {
				Ok(transactions) => transactions,
				Err(err) => {
					eprintln!("{err}");
					continue
				},
			};

			for transaction in account_transactions.into_iter() {
				let observed_transactions = institution.observed_transactions
					.entry(account_id.to_string())
					.or_default();

				let is_unseen = observed_transactions.insert(transaction.id.clone());
				if is_unseen {
					transactions.push(transaction);
				}
			}
		}
	}

	println!("{:#?}", transactions);

	let save_config = Config {
		client_credentials: Some(client_credentials),
		token: Some(token),
		selected_institutions,
	};

	confy::store(APP_NAME, Some("config"), save_config)?;

	Ok(())
}
