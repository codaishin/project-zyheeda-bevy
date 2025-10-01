use std::fmt::Display;
use tracing::{error, field::display, warn};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Default)]
pub struct Logger;

impl Log for Logger {
	fn log_warning<TDetails>(&self, label: &str, details: TDetails)
	where
		TDetails: Display,
	{
		let details = display(details);
		warn!(details, "{label}");
	}

	fn log_error<TDetails>(&self, label: &str, details: TDetails)
	where
		TDetails: Display,
	{
		let details = display(details);
		error!(details, "{label}");
	}
}

pub trait Log {
	fn log_warning<TDetails>(&self, label: &str, details: TDetails)
	where
		TDetails: Display + 'static;
	fn log_error<TDetails>(&self, label: &str, details: TDetails)
	where
		TDetails: Display + 'static;
}
