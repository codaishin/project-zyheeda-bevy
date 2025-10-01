use crate::errors::{ErrorData, Level};
use bevy::ecs::system::In;
use tracing::{error, field::display, warn};

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
				output(error);
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

fn output<TError>(error: TError)
where
	TError: ErrorData,
{
	let label = TError::label();
	let context = display(error.context());

	match error.level() {
		Level::Error => {
			error!(context, "{label}");
		}
		Level::Warning => {
			warn!(context, "{label}");
		}
	}
}
