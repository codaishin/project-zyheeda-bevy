use crate::{
	error_logger::{GlobalErrorLogger, Log},
	errors::ErrorData,
};
use bevy::ecs::system::In;

pub struct OnError;

impl OnError {
	fn void() {}

	pub fn log<TIn>(result: In<TIn>, error_logger: GlobalErrorLogger) -> TIn::TOut
	where
		TIn: OnErrorLogAndReturn<TValue = ()>,
	{
		Self::log_and_return(Self::void)(result, error_logger)
	}

	pub fn log_and_return<TIn>(
		fallback: fn() -> TIn::TValue,
	) -> impl Fn(In<TIn>, GlobalErrorLogger) -> TIn::TOut
	where
		TIn: OnErrorLogAndReturn,
	{
		move |In(result), error_logger| result.process(fallback, &error_logger)
	}
}

pub trait OnErrorLogAndReturn {
	type TOut;
	type TValue;

	fn process(
		self,
		fallback: fn() -> Self::TValue,
		error_logger: &GlobalErrorLogger,
	) -> Self::TOut;
}

impl<TValue, TError> OnErrorLogAndReturn for Result<TValue, TError>
where
	TError: ErrorData,
{
	type TOut = TValue;
	type TValue = TValue;

	fn process(
		self,
		fallback: fn() -> Self::TValue,
		error_logger: &GlobalErrorLogger,
	) -> Self::TOut {
		match self {
			Ok(value) => value,
			Err(error) => {
				error_logger.log(error);
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

	fn process(
		self,
		fallback: fn() -> Self::TValue,
		error_logger: &GlobalErrorLogger,
	) -> Self::TOut {
		self.into_iter()
			.map(move |result| result.process(fallback, error_logger))
			.collect()
	}
}
