use tracing::{error, warn};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Default)]
pub struct Logger;

impl Log for Logger {
	fn log_warning<TError>(&self, value: TError)
	where
		TError: Into<String>,
	{
		warn!("{}", value.into())
	}

	fn log_error<TError>(&self, value: TError)
	where
		TError: Into<String>,
	{
		error!("{}", value.into());
	}
}

pub trait Log {
	fn log_warning<TError>(&self, value: TError)
	where
		TError: Into<String> + 'static;
	fn log_error<TError>(&self, value: TError)
	where
		TError: Into<String> + 'static;
}
