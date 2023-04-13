use color_eyre::eyre;
use inquire::{MultiSelect};
use crate::institution::Institution;

pub struct InstitutionSelect {
	available_institutions: Vec<Institution>,
}

impl<'a> InstitutionSelect {
	pub fn new(available_institutions: Vec<Institution>) -> InstitutionSelect {
		InstitutionSelect {
			available_institutions,
		}
	}

	pub fn prompt(self) -> eyre::Result<Vec<Institution>> {
		let multi_select = MultiSelect::new("Choose institutions you have accounts with", self.available_institutions)
			.with_keep_filter(false);

		let chosen_institutions = multi_select.prompt()?;

		Ok(chosen_institutions)
	}
}
