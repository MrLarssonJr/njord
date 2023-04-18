mod nordigen;
mod interactions;
mod matcher;

use color_eyre::eyre;
use once_cell::sync::Lazy;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use crate::matcher::match_transactions;
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

	println!("{matched_transactions:#?}");

	Ok(())
}
