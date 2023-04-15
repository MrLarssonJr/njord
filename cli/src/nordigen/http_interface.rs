use color_eyre::eyre;
use color_eyre::eyre::eyre;
use reqwest::blocking::Client;
use reqwest::header::HeaderValue;
use reqwest::Url;
use serde::{Deserialize, Serialize};


#[derive(Debug, Deserialize)]
pub struct PaginatedResult<E> {
	pub count: i64,
	pub next: Option<String>,
	pub previous: Option<String>,
	pub results: Vec<E>,
}

fn build_url(endpoint: &str, id: Option<&str>) -> eyre::Result<Url> {
	let mut url = Url::parse("https://ob.nordigen.com/api/v2")?;

	{
		let mut path_segments = url.path_segments_mut()
			.map_err(|_| eyre!("url not base"))?;

		for segment in endpoint.split("/") {
			path_segments.push(segment);
		}

		if let Some(id) = id {
			path_segments.push(id);
		}
		path_segments.push("");
	}

	Ok(url)
}

fn post<Req: Serialize, Res: for<'de> Deserialize<'de>>(client: &Client, endpoint: &str, body: &Req, token: Option<&str>) -> eyre::Result<Res> {
	let req = {
		let url = build_url(endpoint, None)?;
		let mut builder = client.post(url)
			.header("Content-Type", HeaderValue::from_static("application/json"))
			.json(body);

		if let Some(token) = token {
			builder = builder.bearer_auth(token);
		}

		builder.build()?
	};

	let res = {
		let res = client.execute(req)?;
		let status = res.status();

		if !status.is_success() {
			eprintln!("{status}");
			let body = res.text()?;
			eprintln!("{body}");
			return Err(eyre!("[nordigen post] non OK response"));
		}

		res.json()?
	};

	Ok(res)
}

fn get<Res: for<'de> Deserialize<'de>>(client: &Client, endpoint: &str, token: Option<&str>, id: Option<&str>) -> eyre::Result<Res> {
	let req = {
		let url = build_url(endpoint, id)?;
		let mut builder = client.get(url);

		if let Some(token) = token {
			builder = builder.bearer_auth(token);
		}

		builder.build()?
	};

	let res = {
		let res = client.execute(req)?;
		let status = res.status();

		if !status.is_success() {
			eprintln!("{status}");
			let body = res.text()?;
			eprintln!("{body}");
			return Err(eyre!("[nordigen get] non OK response"));
		}

		res.json()?
	};

	Ok(res)
}

pub mod accounts {
	pub mod transactions {
		use chrono::NaiveDate;
		use color_eyre::eyre;
		use reqwest::blocking::Client;
		use rust_decimal::Decimal;
		use serde::Deserialize;
		use crate::nordigen::http_interface;

		#[derive(Debug, Deserialize)]
		pub struct GetResponseBody {
			pub transactions: Transactions,
		}

		#[derive(Debug, Deserialize)]
		pub struct Transactions {
			pub booked: Vec<BookedTransaction>
		}

		#[derive(Debug, Deserialize)]
		pub struct BookedTransaction {
			#[serde(rename = "valueDate")]
			pub value_date: NaiveDate,
			#[serde(rename = "transactionAmount")]
			pub transaction_amount: Amount,
			#[serde(rename = "transactionId")]
			pub transaction_id: String,
			#[serde(rename = "additionalInformation")]
			pub additional_information: String,
		}

		#[derive(Debug, Deserialize)]
		pub struct Amount {
			pub amount: Decimal,
			pub currency: String,
		}

		pub fn get(client: &Client, token: &str, account_id: &str) -> eyre::Result<GetResponseBody> {
			let endpoint = format!("accounts/{account_id}/transactions");
			http_interface::get(client, &endpoint, Some(token), None)
		}
	}
}

pub mod agreements {
	pub mod enduser {
		use std::num::NonZeroU64;
		use chrono::{DateTime, Local};
		use serde::{Deserialize, Serialize};

		#[derive(Debug, Deserialize)]
		pub struct GetResponseBody {
			pub id: String,
			pub created: DateTime<Local>,
			pub institution_id: String,
			pub max_historical_days: u64,
			pub access_valid_for_days: u64,
			pub access_scope: Vec<String>,
			pub accepted: Option<DateTime<Local>>,
		}

		#[derive(Debug, Serialize)]
		pub struct PostRequestBody {
			pub institution_id: String,
			pub max_historical_days: NonZeroU64,
			pub access_valid_for_days: NonZeroU64,
		}


		#[derive(Debug, Deserialize)]
		pub struct PostResponseBody {
			pub id: String,
			pub created: DateTime<Local>,
			pub institution_id: String,
			pub max_historical_days: u64,
			pub access_valid_for_days: u64,
			pub access_scope: Vec<String>,
			pub accepted: Option<DateTime<Local>>,
		}

	}
}

pub mod institutions {
	use color_eyre::eyre;
	use reqwest::blocking::Client;
	use serde::Deserialize;
	use crate::nordigen::http_interface;

	#[derive(Debug, Deserialize)]
	pub struct GetResponseBody {
		pub id: String,
		pub name: String,
		pub bic: Option<String>,
		pub transaction_total_days: Option<String>,
		pub countries: Vec<String>,
		pub logo: String,
	}

	pub fn list(client: &Client, token: &str) -> eyre::Result<Vec<GetResponseBody>> {
		http_interface::get(client, "institutions", Some(token), None)
	}
}

pub mod requisitions {
	use chrono::{DateTime, Local};
	use color_eyre::eyre;
	use reqwest::blocking::Client;
	use serde::{Deserialize, Serialize};
	use crate::nordigen::http_interface;

	#[derive(Debug, Deserialize)]
	pub struct GetResponseBody {
		pub id: String,
		pub created: DateTime<Local>,
		pub status: String,
		pub institution_id: String,
		pub agreement: Option<String>,
		pub accounts: Vec<String>,
		pub link: String,
	}

	pub fn get(client: &Client, token: &str, id: &str) -> eyre::Result<GetResponseBody> {
		http_interface::get(client, "requisitions", Some(token), Some(id))
	}


	#[derive(Debug, Serialize)]
	pub struct PostRequestBody<'a> {
		pub redirect: &'a str,
		pub institution_id: &'a str,
	}

	#[derive(Serialize, Deserialize)]
	pub struct PostResponseBody {
		pub id: String,
		pub created: DateTime<Local>,
		pub status: String,
		pub institution_id: String,
		pub agreement: Option<String>,
		pub accounts: Vec<String>,
		pub link: String,
	}

	pub fn post(client: &Client, token: &str, body: &PostRequestBody) -> eyre::Result<PostResponseBody> {
		http_interface::post(client, "requisitions", body, Some(token))
	}
}

pub mod token {
	pub mod new {
		use color_eyre::eyre;
		use reqwest::blocking::Client;
		use serde::{Deserialize, Serialize};
		use crate::nordigen::client_credentials::ClientCredentials;
		use crate::nordigen::http_interface;

		#[derive(Debug, Serialize)]
		pub struct PostRequestBody<'a> {
			pub secret_id: &'a str,
			pub secret_key: &'a str,
		}

		impl<'a> From<&'a ClientCredentials> for PostRequestBody<'a> {
			fn from(client_credentials: &'a ClientCredentials) -> Self {
				PostRequestBody {
					secret_id: &client_credentials.id,
					secret_key: &client_credentials.secret,
				}
			}
		}

		#[derive(Debug, Deserialize)]
		pub struct PostResponseBody {
			pub access: String,
			pub access_expires: i64,
			pub refresh: String,
			pub refresh_expires: i64,
		}

		pub fn post(client: &Client, body: &PostRequestBody) -> eyre::Result<PostResponseBody> {
			http_interface::post(client, "token/new", body, None)
		}
	}

	pub mod refresh {
		use color_eyre::eyre;
		use reqwest::blocking::Client;
		use serde::{Deserialize, Serialize};
		use crate::nordigen::http_interface;

		#[derive(Debug, Serialize)]
		pub struct PostRequestBody<'a> {
			pub refresh: &'a str,
		}

		#[derive(Serialize, Deserialize)]
		pub struct PostResponseBody {
			pub access: String,
			pub access_expires: i64,
		}

		pub fn post(client: &Client, body: &PostRequestBody) -> eyre::Result<PostResponseBody> {
			http_interface::post(client, "token/refresh", body, None)
		}
	}
}
