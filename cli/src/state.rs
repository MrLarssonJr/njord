use std::error::Error;
use std::fmt::{Display, Formatter};
use std::iter::once;
use std::num::NonZeroU64;
use std::str::FromStr;
use chrono::{DateTime, Duration, Local, NaiveTime, Weekday};
use inquire::{DateSelect, Select};
use crate::config::Config;
use crate::HTTP_CLIENT;
use crate::nordigen::NordigenClientContext;

#[derive(Debug, Copy, Clone)]
enum Action {
	ModifySettings,
	ModifyAvailableAuthorizations,
	GetTransactions,
	Exit,
}

impl Action {
	fn perform(self, state: &mut State) -> Result<bool, Box<dyn Error>> {
		match self {
			Action::ModifySettings => Action::modify_settings(state),
			Action::ModifyAvailableAuthorizations => Action::modify_available_authorizations(state),
			Action::Exit => Ok(false),
			Action::GetTransactions => {}
		}
	}

	fn modify_settings(state: &mut State) -> Result<bool, Box<dyn Error>> {
		#[derive(Debug, Copy, Clone)]
		enum SubAction {
			SelectedInstitutions,
			ClientCredentials,
		}

		impl Display for SubAction {
			fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
				match self {
					SubAction::SelectedInstitutions => write!(f, "Select institutions"),
					SubAction::ClientCredentials => write!(f, "Update client credentials"),
				}
			}
		}

		let possible_actions = vec![SubAction::SelectedInstitutions, SubAction::ClientCredentials];
		let selected_action = Select::new("Which setting do you wish to change?", possible_actions)
			.prompt_skippable()?;

		let Some(selected_action) = selected_action else {
			return Ok(true);
		};

		match selected_action {
			SubAction::SelectedInstitutions => state.config.select_institutions()?,
			SubAction::ClientCredentials => state.config.set_client_credentials()?,
		};

		Ok(true)
	}

	fn modify_available_authorizations(state: &mut State) -> Result<bool, Box<dyn Error>> {
		#[derive(Debug, Copy, Clone)]
		enum SubAction {
			Delete,
			Add,
		}

		impl Display for SubAction {
			fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
				match self {
					SubAction::Delete => write!(f, "Delete available authorizations"),
					SubAction::Add => write!(f, "Add available authorization"),
				}
			}
		}

		let institutions = state.config.institutions.iter().cloned().collect();
		let mut ctx = NordigenClientContext::new(&mut state.config, &HTTP_CLIENT)?;
		let euas = ctx.list_euas()?;

		println!("Currently available authorizations");
		for eua in euas.iter() {
			println!("{eua}");
		}

		let possible_actions = vec![SubAction::Add, SubAction::Delete];
		let selected_action = Select::new("How would you like to modify the available authorizations?", possible_actions)
			.prompt_skippable()?;

		let Some(selected_action) = selected_action else {
			return Ok(true);
		};

		match selected_action {
			SubAction::Delete => {
				let selected_eua = Select::new("Which?", euas)
					.prompt()?;

				if selected_eua.accepted.is_none() {
					ctx.delete_eua(&selected_eua.id)?;
				} else {
					println!("Can't delete accepted authorizations");
				}

			}
			SubAction::Add => {
				let institution_id = Select::new("For which institution?", institutions)
					.prompt()?
					.id;

				let access_back_to = DateSelect::new("How far would you like to access?")
					.with_min_date((Local::now() - Duration::days(730)).date_naive())
					.with_max_date((Local::now() - Duration::days(1)).date_naive())
					.with_week_start(Weekday::Mon)
					.with_starting_date((Local::now() - Duration::days(90)).date_naive())
					.prompt()?
					.and_time(NaiveTime::default())
					.and_local_timezone(Local)
					.latest()
					.ok_or("Invalid date")?;

				let max_historical_days: u64 = (Local::now() - dbg!(access_back_to)).num_days().try_into()?;
				let max_historical_days: NonZeroU64 = max_historical_days.try_into()?;

				let res = ctx.create_eua(institution_id, Some(dbg!(max_historical_days)), None)?;

				println!("Created");
				println!("{res}");
			}
		}

		Ok(true)
	}

	fn get_transactions(state: &mut State) -> Result<bool, Box<dyn Error>> {
		let reqs = NordigenClientContext::new(&mut state.config, &HTTP_CLIENT)?.list_requisitions()?;

		#[derive(Debug, Copy, Clone)]
		enum SelectedReq<'a> {
			Existing(&'a Requisition),
			New
		}

		impl Display for SelectedReq {
			fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
				match self {
					SelectedReq::Existing(req) => write!(f, "{req}"),
					SelectedReq::New => write!("new"),
				}
			}
		}

		for institution in state.config.institutions.iter() {
			let options: Vec<_> = reqs.iter()
				.filter(|r| r.institution_id == institution.id)
				.map(SelectedReq::Existing)
				.chain(once(SelectedReq::New))
				.collect();

			let selection = Select::new("Select a requisition", options)
				.prompt()?;

			
		}

		todo!()

	}
}

impl Display for Action {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Action::ModifySettings => write!(f, "Modify settings"),
			Action::ModifyAvailableAuthorizations => write!(f, "Modify available authorizations"),
			Action::Exit => write!(f, "Exit"),
			Action::GetTransactions => write!(f, "Get Transactions"),
		}
	}
}

pub struct State {
	config: Config,
}

impl State {
	pub fn new(config: Config) -> Result<State, Box<dyn Error>> {
		Ok(State {
			config,
		})
	}

	pub fn prompt(&mut self) -> Result<bool, Box<dyn Error>> {
		let possible_actions = vec![
			Action::ModifyAvailableAuthorizations,
			Action::ModifySettings,
			Action::Exit,
		];

		let selected_action = Select::new("What would you like to do?", possible_actions)
			.prompt()?;

		let res = selected_action.perform(self)?;

		self.config.save()?;

		Ok(res)
	}
}

#[derive(Debug)]
pub struct EndUserAgreement {
	pub id: String,
	pub created: DateTime<Local>,
	pub institution_id: String,
	pub max_historical_days: u64,
	pub access_valid_for_days: u64,
	pub access_scope: Vec<String>,
	pub accepted: Option<DateTime<Local>>,
}

impl Display for EndUserAgreement {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let EndUserAgreement { created, institution_id, max_historical_days, access_valid_for_days, .. } = self;
		write!(f, "{institution_id}\t{max_historical_days}\t{created}\t{access_valid_for_days}")
	}
}

#[derive(Debug)]
pub struct Requisition {
	pub id: String,
	pub created: DateTime<Local>,
	pub status: RequisitionStatus,
	pub institution_id: String,
	pub agreement: Option<String>,
	pub accounts: Vec<String>,
	pub link: String,
}

impl Display for Requisition {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let Requisition { created, institution_id, .. } = self;
		write!(f, "{institution_id}\t{created}")
	}
}

#[derive(Debug)]
pub enum RequisitionStatus {
	Created,
	GivingConsent,
	UndergoingAuthentication,
	Rejected,
	SelectingAccounts,
	GrantingAccess,
	Linked,
	Suspended,
	Expired,
}

impl FromStr for RequisitionStatus {
	type Err = Box<dyn Error>;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		use RequisitionStatus::*;

		match s {
			"CR" => Ok(Created),
			"GC" => Ok(GivingConsent),
			"UA" => Ok(UndergoingAuthentication),
			"RJ" => Ok(Rejected),
			"SA" => Ok(SelectingAccounts),
			"GA" => Ok(GrantingAccess),
			"LN" => Ok(Linked),
			"SU" => Ok(Suspended),
			"EX" => Ok(Expired),
			_ => Err("invalid code".into())
		}
	}
}
