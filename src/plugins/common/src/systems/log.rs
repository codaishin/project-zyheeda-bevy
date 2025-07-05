use crate::errors::{Error, Level};
use bevy::ecs::system::In;
use tracing::{error, warn};

pub struct OnError;

impl OnError {
	fn void() {}

	pub fn log<TError>(result: In<Result<(), TError>>)
	where
		Error: From<TError>,
	{
		Self::log_and_fallback(Self::void)(result)
	}

	pub fn log_and_fallback<TValue, TError>(
		fallback: fn() -> TValue,
	) -> impl Fn(In<Result<TValue, TError>>) -> TValue
	where
		Error: From<TError>,
	{
		move |In(result)| match result {
			Ok(value) => value,
			Err(error) => {
				output(error);
				fallback()
			}
		}
	}

	pub fn log_many<TError, TResults>(In(results): In<TResults>)
	where
		Error: From<TError> + From<TResults::Error>,
		TResults: TryInto<Vec<Result<(), TError>>>,
	{
		match results.try_into() {
			Err(error) => output(Error::from(error)),
			Ok(results) => {
				for error in results.into_iter().filter_map(|result| result.err()) {
					output(error);
				}
			}
		}
	}
}

fn output<TError>(error: TError)
where
	Error: From<TError>,
{
	let Error { msg, lvl } = Error::from(error);
	match lvl {
		Level::Error => error!(msg),
		Level::Warning => warn!(msg),
	}
}
