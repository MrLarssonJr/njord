use serde::{Deserialize, Serialize};
use chrono::{DateTime, Duration, Local};
use color_eyre::eyre;
use crate::client_credentials::ClientCredentials;
use crate::HTTP_CLIENT;
use crate::nordigen::http_interface;

#[derive(Debug, Serialize, Deserialize)]
pub struct Token {
	access: TokenPart,
	refresh: TokenPart,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct TokenPart {
	secret: String,
	expires_at: DateTime<Local>,
}

impl Token {
	pub fn new(client_credentials: &ClientCredentials) -> eyre::Result<Token> {
		let start = Local::now();
		let body = client_credentials.into();
		let res = http_interface::token::new::post(&HTTP_CLIENT, &body)?;

		Ok(Token {
			access: TokenPart { secret: res.access, expires_at: start + Duration::seconds(res.access_expires) },
			refresh: TokenPart { secret: res.refresh, expires_at: start + Duration::seconds(res.refresh_expires) },
		})
	}

	pub fn get_access_token(&mut self, client_credentials: &ClientCredentials) -> eyre::Result<&str> {
		let time_til_access_expiry = Local::now() - self.access.expires_at;
		let access_ok = time_til_access_expiry < Duration::seconds(30);

		if access_ok {
			return Ok(&self.access.secret);
		}

		let start = Local::now();
		let body = http_interface::token::refresh::PostRequestBody {
			refresh: self.get_refresh_token(client_credentials)?,
		};
		let res = http_interface::token::refresh::post(&HTTP_CLIENT, &body)?;
		self.access.secret = res.access;
		self.access.expires_at = start + Duration::seconds(res.access_expires);

		return Ok(&self.access.secret);
	}

	pub fn get_refresh_token(&mut self, client_credentials: &ClientCredentials) -> eyre::Result<&str> {
		let time_til_refresh_expiry = Local::now() - self.refresh.expires_at;
		let refresh_ok = time_til_refresh_expiry < Duration::seconds(30);

		if refresh_ok {
			return Ok(&self.refresh.secret);
		}

		*self = Token::new(client_credentials)?;

		return Ok(&self.refresh.secret);
	}
}
