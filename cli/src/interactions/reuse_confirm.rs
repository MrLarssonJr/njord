use color_eyre::eyre;
use inquire::{Confirm};
use crate::institution::Institution;

pub struct ReuseConfirm<'a> {
	selected: &'a [Institution],
}

impl<'a> ReuseConfirm<'a> {
	pub fn new(selected: &'a [Institution]) -> ReuseConfirm {
		ReuseConfirm {
			selected
		}
	}

	pub fn prompt(self) -> eyre::Result<bool> {
		if self.selected.is_empty() {
			return Ok(false);
		}

		println!("These institutions were selected last time:");

		for institution in self.selected {
			println!("{institution}");
		}

		Ok(Confirm::new("Reuse them?").prompt()?)
	}
}
