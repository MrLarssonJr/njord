use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientCredentials {
	pub id: String,
	pub secret: String,
}
