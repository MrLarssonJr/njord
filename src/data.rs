use std::error::Error;
use mongodb::bson::doc;
use mongodb::Client;
use mongodb::options::{ClientOptions, FindOneOptions, ReplaceOptions};
use serde::{Deserialize, Serialize};
use crate::env;

#[derive(Clone)]
pub struct Database {
	handle: mongodb::Database
}

impl Database {
	pub async fn new() -> Database {
		let connection_string = env::must_get("MONGO_URL");
		let client_options = ClientOptions::parse(connection_string)
			.await
			.expect("variable MONGO_URL in environment must be a valid mongodb connection string");
		let client = Client::with_options(client_options)
			.expect("must be able to connect to cluster");
		let database = client.database("njord");

		Database {
			handle: database
		}
	}

	pub async fn inc_and_get_count<K: AsRef<str>>(&self, key: &K) -> Result<u64, Box<dyn Error>> {
		let count = {
		let filter = doc! { "target": key.as_ref() };
			self.handle
				.collection::<Count>("count")
				.find_one(filter, FindOneOptions::default())
				.await?
				.map(|Count { count, ..} | count)
				.unwrap_or(0)
		};

		let count = count + 1;

		{
			let count = Count {
				target: key.as_ref().to_string(),
				count,
			};

			let filter = doc! { "target": key.as_ref() };
			let options = ReplaceOptions::builder()
				.upsert(true)
				.build();

			self.handle
				.collection::<Count>("count")
				.replace_one(filter, count, options)
				.await?;
		}

		Ok(count)
	}
}

#[derive(Serialize, Deserialize)]
pub struct Count {
	target: String,
	count: u64,
}
