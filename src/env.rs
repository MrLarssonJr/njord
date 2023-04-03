use std::ffi::OsStr;
use std::fmt::Display;

pub fn must_get<K: AsRef<OsStr> + Display + Copy>(key: K) -> String {
	match std::env::var(key) {
		Ok(val) => val,
		Err(std::env::VarError::NotPresent) => panic!("expected {key} to be present in environment"),
		Err(std::env::VarError::NotUnicode(_)) => panic!("expected value for key {key} in environment to be valid UTF-8"),
	}
}
