use crate::errors::ErrorData;
use bevy::ecs::system::In;

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
	TError: ErrorData,
{
	type TOut = TValue;
	type TValue = TValue;

	fn process(self, fallback: fn() -> Self::TValue) -> Self::TOut {
		match self {
			Ok(value) => value,
			Err(error) => {
				output::log(error);
				fallback()
			}
		}
	}
}

impl<TValue, TError> OnErrorLogAndReturn for Vec<Result<TValue, TError>>
where
	TError: ErrorData,
{
	type TOut = Vec<TValue>;
	type TValue = TValue;

	fn process(self, fallback: fn() -> Self::TValue) -> Self::TOut {
		self.into_iter()
			.map(|result| result.process(fallback))
			.collect()
	}
}

#[cfg(not(test))]
pub(crate) mod output {
	use super::*;
	use crate::errors::Level;
	use tracing::{error, field::display, warn};

	pub(crate) fn log<TError>(error: TError)
	where
		TError: ErrorData,
	{
		let level = error.level();
		let label = TError::label();
		let details = display(error.into_details());

		match level {
			Level::Error => {
				error!(details, "{label}");
			}
			Level::Warning => {
				warn!(details, "{label}");
			}
		}
	}
}

#[cfg(test)]
pub(crate) mod output {
	use super::*;

	pub(crate) fn log<TError>(_: TError)
	where
		TError: ErrorData,
	{
	}
}
