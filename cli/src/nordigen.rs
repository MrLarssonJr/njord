use std::error::Error;
use std::num::NonZeroU64;
use chrono::{Duration, Local};
use reqwest::blocking::{Client};
use crate::config::{ClientCredentials, Config, Institution, Token};
use crate::state::{EndUserAgreement, Requisition};

pub struct NordigenClientContext<'a> {
	http_client: &'a Client,
	token: &'a mut Option<Token>,
	client_credentials: &'a ClientCredentials,
}

impl<'a> NordigenClientContext<'a> {
	pub fn new(config: &'a mut Config, http_client: &'a Client) -> Result<NordigenClientContext<'a>, Box<dyn Error>> {
		let client = NordigenClientContext {
			http_client,
			token: &mut config.token,
			client_credentials: &config.client_credentials,
		};

		Ok(client)
	}

	pub fn available_institutions(&mut self) -> Result<Vec<Institution>, Box<dyn Error>> {
		let token = get_token(self.http_client, self.client_credentials, self.token)?;

		let res = http_interface::institutions::get(self.http_client, token)?;
		let res = res.into_iter()
			.map(|res| res.into())
			.collect();

		Ok(res)
	}

	pub fn list_requisitions(&mut self) -> Result<Vec<Requisition>, Box<dyn Error>> {
		let token = get_token(self.http_client, self.client_credentials, self.token)?;

		let res = http_interface::requisitions::get(self.http_client, token)?;
		let res = res.results.into_iter()
			.map(|res| res.try_into())
			.collect::<Result<_, _>>()?;

		Ok(res)
	}

	pub fn list_euas(&mut self) -> Result<Vec<EndUserAgreement>, Box<dyn Error>> {
		let token = get_token(self.http_client, self.client_credentials, self.token)?;

		let res = http_interface::agreements::enduser::get(self.http_client, token)?;
		let res = res.results.into_iter()
			.map(|res| res.into())
			.collect();

		Ok(res)
	}

	pub fn create_eua(&mut self, institution_id: impl Into<String>, max_historical_days: Option<NonZeroU64>, access_valid_for_days: Option<NonZeroU64>) -> Result<EndUserAgreement, Box<dyn Error>> {
		let token = get_token(self.http_client, self.client_credentials, self.token)?;

		let body = http_interface::agreements::enduser::PostRequestBody {
			institution_id: institution_id.into(),
			max_historical_days: max_historical_days.unwrap_or(NonZeroU64::new(1).unwrap()),
			access_valid_for_days: access_valid_for_days.unwrap_or(NonZeroU64::new(1).unwrap()),
		};

		let res = http_interface::agreements::enduser::post(self.http_client, token, &body)?
			.into();


		Ok(res)
	}

	pub fn delete_eua(&mut self, id: &str) -> Result<(), Box<dyn Error>> {
		let token = get_token(self.http_client, self.client_credentials, self.token)?;
		http_interface::agreements::enduser::delete(self.http_client, token, id)?;
		Ok(())
	}
}

fn new_token(client: &Client, client_credentials: &ClientCredentials) -> Result<Token, Box<dyn Error>> {
	let start = Local::now();
	let body = client_credentials.into();
	let res = http_interface::token::new::post(client, &body)?;
	Ok(res.into_token(start))
}

fn get_token<'t>(http_client: &Client, client_credentials: &ClientCredentials, token: &'t mut Option<Token>) -> Result<&'t str, Box<dyn Error>> {
	// 1. If no token exists, create one and return it
	// 2. If it exist, and is valid, return it
	// 3. If it exist, and is invalid, refresh it and return it
	// 4. If it exist, but refresh is also invalid, create a new one and return it

	// 1.
	let Some(token) = token else {
		let new_token = new_token(http_client, client_credentials)?;
		let token = token.insert(new_token);

		return Ok(&token.access.secret);
	};

	// 2.
	let time_til_access_expiry = Local::now() - token.access.expires_at;
	let access_ok = time_til_access_expiry < Duration::seconds(30);

	if access_ok {
		return Ok(&token.access.secret);
	}

	// 3.
	let time_til_refresh_expiry = Local::now() - token.refresh.expires_at;
	let refresh_ok = time_til_refresh_expiry < Duration::seconds(30);

	if refresh_ok {
		let start = Local::now();
		let body = (&*token).into();
		let res = http_interface::token::refresh::post(http_client, &body)?;
		let new_access_token = res.into_token_part(start);
		token.access = new_access_token;

		return Ok(&token.access.secret);
	}

	// 4.
	let new_token = new_token(http_client, client_credentials)?;
	*token = new_token;

	return Ok(&token.access.secret);
}

mod http_interface {
	use std::error::Error;
	use reqwest::blocking::Client;
	use reqwest::header::HeaderValue;
	use reqwest::{Url};
	use serde::{Deserialize, Serialize};


	#[derive(Debug, Deserialize)]
	pub struct PaginatedResult<E> {
		pub count: i64,
		pub next: Option<String>,
		pub previous: Option<String>,
		pub results: Vec<E>,
	}

	fn build_url(endpoint: &str, id: Option<&str>) -> Result<Url, Box<dyn Error>> {
		let mut url = Url::parse("https://ob.nordigen.com/api/v2")?;

		{
			let mut path_segments = url.path_segments_mut()
				.map_err(|_| "url not base")?;

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

	fn post<Req: Serialize, Res: for<'de> Deserialize<'de>>(client: &Client, endpoint: &str, body: &Req, token: Option<&str>) -> Result<Res, Box<dyn Error>> {
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
				return Err("[nordigen post] non OK response".into());
			}

			res.json()?
		};

		Ok(res)
	}

	fn get<Res: for<'de> Deserialize<'de>>(client: &Client, endpoint: &str, token: Option<&str>) -> Result<Res, Box<dyn Error>> {
		let req = {
			let url = build_url(endpoint, None)?;
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
				return Err("[nordigen get] non OK response".into());
			}

			res.json()?
		};

		Ok(res)
	}

	fn delete(client: &Client, endpoint: &str, token: Option<&str>, id: &str) -> Result<(), Box<dyn Error>> {
		let req = {
			let url = build_url(endpoint, Some(id))?;
			let mut builder = client.delete(url);

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
				return Err("[nordigen delete] non OK response".into());
			}
		};

		Ok(res)
	}

	pub mod agreements {
		pub mod enduser {
			use std::error::Error;
			use std::num::NonZeroU64;
			use chrono::{DateTime, Local};
			use reqwest::blocking::Client;
			use serde::{Deserialize, Serialize};
			use crate::nordigen::http_interface;
			use crate::nordigen::http_interface::PaginatedResult;
			use crate::state::EndUserAgreement;

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

			impl Into<EndUserAgreement> for GetResponseBody {
				fn into(self) -> EndUserAgreement {
					EndUserAgreement {
						id: self.id,
						created: self.created,
						institution_id: self.institution_id,
						max_historical_days: self.max_historical_days,
						access_valid_for_days: self.access_valid_for_days,
						access_scope: self.access_scope,
						accepted: self.accepted,
					}
				}
			}

			pub fn get(client: &Client, token: &str) -> Result<PaginatedResult<GetResponseBody>, Box<dyn Error>> {
				http_interface::get(client, "agreements/enduser", Some(token))
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

			impl Into<EndUserAgreement> for PostResponseBody {
				fn into(self) -> EndUserAgreement {
					EndUserAgreement {
						id: self.id,
						created: self.created,
						institution_id: self.institution_id,
						max_historical_days: self.max_historical_days,
						access_valid_for_days: self.access_valid_for_days,
						access_scope: self.access_scope,
						accepted: self.accepted,
					}
				}
			}

			pub fn post(client: &Client, token: &str, body: &PostRequestBody) -> Result<PostResponseBody, Box<dyn Error>> {
				http_interface::post(client, "agreements/enduser", body,Some(token))
			}

			pub fn delete(client: &Client, token: &str, id: &str) -> Result<(), Box<dyn Error>> {
				http_interface::delete(client, "agreements/enduser", Some(token), id)
			}
		}
	}

	pub mod institutions {
		use std::error::Error;
		use reqwest::blocking::Client;
		use serde::Deserialize;
		use crate::config::{Institution};
		use crate::nordigen::{http_interface};

		#[derive(Debug, Deserialize)]
		pub struct GetResponseBody {
			pub id: String,
			pub name: String,
			pub bic: Option<String>,
			pub transaction_total_days: Option<String>,
			pub countries: Vec<String>,
			pub logo: String,
		}

		impl Into<Institution> for GetResponseBody {
			fn into(self) -> Institution {
				Institution {
					id: self.id,
					name: self.name,
					countries: self.countries,
				}
			}
		}

		pub fn get(client: &Client, token: &str) -> Result<Vec<GetResponseBody>, Box<dyn Error>> {
			http_interface::get(client, "institutions", Some(token))
		}
	}

	pub mod requisitions {
		use std::error::Error;
		use chrono::{DateTime, Local};
		use reqwest::blocking::Client;
		use serde::{Deserialize, Serialize};
		use crate::nordigen::http_interface::PaginatedResult;
		use crate::nordigen::http_interface;
		use crate::state::Requisition;

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

		impl TryFrom<GetResponseBody> for Requisition {
			type Error = Box<dyn Error>;

			fn try_from(res: GetResponseBody) -> Result<Self, Self::Error> {
				Ok(Requisition {
					id: res.id,
					created: res.created,
					status: res.status.parse()?,
					institution_id: res.institution_id,
					agreement: res.agreement,
					accounts: res.accounts,
					link: res.link,
				})
			}
		}

		pub fn get(client: &Client, token: &str) -> Result<PaginatedResult<GetResponseBody>, Box<dyn Error>> {
			http_interface::get(client, "requisitions", Some(token))
		}


		#[derive(Debug, Serialize)]
		pub struct PostRequestBody {
			pub redirect: String,
			pub institution_id: String,
			pub agreement: Option<String>,
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

		impl TryFrom<PostResponseBody> for Requisition {
			type Error = Box<dyn Error>;

			fn try_from(res: PostResponseBody) -> Result<Self, Self::Error> {
				Ok(Requisition {
					id: res.id,
					created: res.created,
					status: res.status.parse()?,
					institution_id: res.institution_id,
					agreement: res.agreement,
					accounts: res.accounts,
					link: res.link,
				})
			}
		}

		pub fn post(client: &Client, token: &str, body: &PostRequestBody) -> Result<PostResponseBody, Box<dyn Error>> {
			http_interface::post(client, "requisitions", body, Some(token))
		}

		pub fn delete(client: &Client, token: &str, id: &str) -> Result<(), Box<dyn Error>> {
			http_interface::delete(client, "requisitions", Some(token), id)
		}
	}

	pub mod token {
		pub mod new {
			use std::error::Error;
			use chrono::{DateTime, Duration, Local};
			use reqwest::blocking::Client;
			use serde::{Deserialize, Serialize};
			use crate::config::{ClientCredentials, Token, TokenPart};
			use crate::nordigen::{http_interface};

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

			impl PostResponseBody {
				pub fn into_token(self, start: DateTime<Local>) -> Token {
					Token {
						access: TokenPart {
							secret: self.access,
							expires_at: start + Duration::seconds(self.access_expires)
						},
						refresh: TokenPart {
							secret: self.refresh,
							expires_at: start + Duration::seconds(self.refresh_expires)
						},
					}
				}
			}

			pub fn post(client: &Client, body: &PostRequestBody) -> Result<PostResponseBody, Box<dyn Error>> {
				http_interface::post(client, "token/new", body, None)
			}
		}

		pub mod refresh {
			use std::error::Error;
			use chrono::{DateTime, Duration, Local};
			use reqwest::blocking::{Client};
			use serde::{Deserialize, Serialize};
			use crate::config::{Token, TokenPart};
			use crate::nordigen::{http_interface};

			#[derive(Debug, Serialize)]
			pub struct PostRequestBody<'a> {
				refresh: &'a str,
			}

			impl<'a> From<&'a Token> for PostRequestBody<'a> {
				fn from(token: &'a Token) -> Self {
					PostRequestBody {
						refresh: &token.refresh.secret,
					}
				}
			}

			#[derive(Serialize, Deserialize)]
			pub struct PostResponseBody {
				pub access: String,
				pub access_expires: i64,
			}

			impl PostResponseBody {
				pub fn into_token_part(self, start: DateTime<Local>) -> TokenPart {
					TokenPart {
						secret: self.access,
						expires_at: start + Duration::seconds(self.access_expires),
					}
				}
			}

			pub fn post(client: &Client, body: &PostRequestBody) -> Result<PostResponseBody, Box<dyn Error>> {
				http_interface::post(client, "token/refresh", body, None)
			}
		}
	}
}

