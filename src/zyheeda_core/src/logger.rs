use std::fmt::Display;
use tracing::{error, field::display, warn};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Default)]
pub struct Logger;

impl Log for Logger {
	fn log_warning<TContext>(&self, label: &str, context: TContext)
	where
		TContext: Display,
	{
		let context = display(context);
		warn!(context, "{label}");
	}

	fn log_error<TContext>(&self, label: &str, context: TContext)
	where
		TContext: Display,
	{
		let context = display(context);
		error!(context, "{label}");
	}
}

pub trait Log {
	fn log_warning<TContext>(&self, label: &str, context: TContext)
	where
		TContext: Display + 'static;
	fn log_error<TContext>(&self, label: &str, context: TContext)
	where
		TContext: Display + 'static;
}
