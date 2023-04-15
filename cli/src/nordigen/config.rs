use serde::{Deserialize, Serialize};
use crate::nordigen::client_credentials::ClientCredentials;
use crate::nordigen::institution::Institution;
use crate::nordigen::token::Token;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
	pub client_credentials: Option<ClientCredentials>,
	pub token: Option<Token>,
	pub selected_institutions: Vec<Institution>,
}

