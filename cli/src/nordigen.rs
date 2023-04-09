use std::error::Error;
use chrono::{Duration, Utc};
use reqwest::blocking::{Client};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use crate::{ClientCredentials, Institution, TokenPart, Token};
use crate::nordigen::dts::{GetInstitutionResponse, PostTokenNewRequest, PostTokenNewResponse};

mod dts {
	use serde::{Deserialize, Serialize};

	#[derive(Debug, Serialize)]
	pub struct PostTokenNewRequest<'a> {
		pub secret_id: &'a str,
		pub secret_key: &'a str,
	}

	#[derive(Debug, Deserialize)]
	pub struct PostTokenNewResponse {
		pub access: String,
		pub access_expires: i64,
		pub refresh: String,
		pub refresh_expires: i64,
	}




	#[derive(Debug, Deserialize)]
	pub struct GetInstitutionResponse {
		pub id: String,
		pub name: String,
		pub bic: Option<String>,
		pub transaction_total_days: Option<String>,
		pub countries: Vec<String>,
		pub logo: String
	}
}

fn build_url(endpoint: &str) -> Result<Url, Box<dyn Error>> {
	let base = Url::parse("https://ob.nordigen.com/api/v2/")?;
	let url = base.join(endpoint)?;
	Ok(url)
}

fn post<Req: Serialize, Res: for<'de> Deserialize<'de>>(client: &Client, endpoint: &str, req: Req) -> Result<Res, Box<dyn Error>> {
	Ok(client.post(build_url(endpoint)?)
		.header("accept", "application/json")
		.header("Content-Type", "application/json")
		.json(&req)
		.send()?
		.json()?)
}

fn get<Res: for<'de> Deserialize<'de>>(client: &Client, endpoint: &str, token: &Token) -> Result<Res, Box<dyn Error>> {
	Ok(client.get(build_url(endpoint)?)
		.header("accept", "application/json")
		.bearer_auth(&token.access.secret)
		.send()?
		.json()?)
}

pub fn new_token(client: &Client, client_credentials: &ClientCredentials) -> Result<Token, Box<dyn Error>> {
	let req = PostTokenNewRequest {
		secret_id: &client_credentials.id,
		secret_key: &client_credentials.secret,
	};

	let start = Utc::now();
	let res: PostTokenNewResponse = post(client, "token/new/", req)?;

	let token_pair = Token {
		access: TokenPart {
			secret: res.access,
			expires_at: start + Duration::seconds(res.access_expires),
		},
		refresh: TokenPart {
			secret: res.refresh,
			expires_at: start + Duration::seconds(res.refresh_expires),
		},
	};

	Ok(token_pair)
}

pub fn get_available_institutions(client: &Client, token: &Token) -> Result<Vec<Institution>, Box<dyn Error>> {
	let res: Vec<GetInstitutionResponse> = get(client, "institutions", token)?;
	let res: Vec<_> = res.into_iter()
		.map(|GetInstitutionResponse { id, name, countries, .. }| Institution { id, name, countries })
		.collect();

	Ok(res)
}
