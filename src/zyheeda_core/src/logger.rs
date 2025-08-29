use std::fmt::Display;
use tracing::{error, warn};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Default)]
pub struct Logger;

impl Log for Logger {
	fn log_warning<TError>(&self, value: TError)
	where
		TError: Display,
	{
		warn!("{value}");
	}

	fn log_error<TError>(&self, value: TError)
	where
		TError: Display,
	{
		error!("{value}");
	}
}

pub trait Log {
	fn log_warning<TError>(&self, value: TError)
	where
		TError: Display + 'static;
	fn log_error<TError>(&self, value: TError)
	where
		TError: Display + 'static;
}
