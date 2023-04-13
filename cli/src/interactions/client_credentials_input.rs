use color_eyre::eyre;
use inquire::Text;
use crate::client_credentials::ClientCredentials;

pub struct ClientCredentialsInput;

impl ClientCredentialsInput {
	pub fn prompt() -> eyre::Result<ClientCredentials> {
		let id = Text::new("Client ID").prompt()?;
		let secret = Text::new("Client secret").prompt()?;

		Ok(ClientCredentials { id, secret })
	}

}
