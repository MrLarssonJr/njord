use serde::{Deserialize, Serialize};
use crate::client_credentials::ClientCredentials;
use crate::institution::Institution;
use crate::token::Token;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
	pub client_credentials: Option<ClientCredentials>,
	pub token: Option<Token>,
	pub selected_institutions: Vec<Institution>,
}

