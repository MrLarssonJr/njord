mod nordigen;
mod config;
mod state;

use std::error::Error;
use once_cell::sync::Lazy;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use crate::config::Config;
use crate::state::State;

pub static APP_NAME: &'static str = "njord";
pub static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
	let mut default_headers = HeaderMap::new();
	default_headers.insert("Accept", HeaderValue::from_static("application/json"));

	Client::builder()
		.default_headers(default_headers)
		.build()
		.expect("unable to build HTTP client")
});

fn main() -> Result<(), Box<dyn Error>> {
	let config = confy::load::<Option<Config>>(APP_NAME, Some("config"))?
		.map(|conf| Ok(conf))
		.unwrap_or_else(|| {
			println!("Found no config");
			Config::new()
		})?;

	let mut state = State::new(config)?;

	while state.prompt()? {}


	Ok(())
}
