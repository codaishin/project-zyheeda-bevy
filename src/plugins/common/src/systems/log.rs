use crate::errors::{Error, Level};
use bevy::ecs::system::In;
use tracing::{error, warn};

pub struct OnError;

impl OnError {
	fn void() {}

	pub fn log<TIn>(result: In<TIn>) -> TIn::TOut
	where
		TIn: OnErrorLogAndReturn<TValue = ()>,
	{
		Self::log_and_return(Self::void)(result)
	}

	pub fn log_and_return<TIn>(fallback: fn() -> TIn::TValue) -> impl Fn(In<TIn>) -> TIn::TOut
	where
		TIn: OnErrorLogAndReturn,
	{
		move |In(result)| result.process(fallback)
	}
}

pub trait OnErrorLogAndReturn {
	type TOut;
	type TValue;

	fn process(self, fallback: fn() -> Self::TValue) -> Self::TOut;
}

impl<TValue, TError> OnErrorLogAndReturn for Result<TValue, TError>
where
	Error: From<TError>,
{
	type TOut = TValue;
	type TValue = TValue;

	fn process(self, fallback: fn() -> Self::TValue) -> Self::TOut {
		match self {
			Ok(value) => value,
			Err(error) => {
				output(error);
				fallback()
			}
		}
	}
}

impl<TValue, TError> OnErrorLogAndReturn for Vec<Result<TValue, TError>>
where
	Error: From<TError>,
{
	type TOut = Vec<TValue>;
	type TValue = TValue;

	fn process(self, fallback: fn() -> Self::TValue) -> Self::TOut {
		self.into_iter()
			.map(|result| result.process(fallback))
			.collect()
	}
}

impl<TError> From<Vec<TError>> for Error
where
	Error: From<TError>,
{
	fn from(errors: Vec<TError>) -> Self {
		Self::Multiple(errors.into_iter().map(Error::from).collect())
	}
}

fn output<TError>(error: TError)
where
	TError: Into<Error>,
{
	match error.into() {
		Error::Single {
			msg,
			lvl: Level::Error,
		} => error!(msg),
		Error::Single {
			msg,
			lvl: Level::Warning,
		} => warn!(msg),
		Error::Multiple(errors) => {
			for error in errors {
				output(error);
			}
		}
	}
}
