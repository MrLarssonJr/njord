use color_eyre::eyre;
use inquire::{Confirm};
use crate::nordigen::institution::Institution;

pub struct AcceptedConfirm<'a> {
	institution: &'a Institution,
}

impl<'a> AcceptedConfirm<'a> {
	pub fn new(institution: &'a Institution) -> AcceptedConfirm {
		AcceptedConfirm {
			institution
		}
	}

	pub fn prompt(self) -> eyre::Result<()> {
		eprintln!("Opening page to authorize access to {} in your browser!", self.institution);

		Confirm::new("Done authorising?")
			.with_default(true)
			.with_help_message("Both a yes or no answer is interpreted s your done")
			.prompt()?;

		Ok(())
	}
}
