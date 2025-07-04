use crate::errors::{Error, Level};
use bevy::ecs::system::In;
use tracing::{error, warn};

pub fn log<TError>(In(result): In<Result<(), TError>>)
where
	Error: From<TError>,
{
	let Err(error) = result else {
		return;
	};

	output(error);
}

pub fn log_with_fallback<TValue, TError>(
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

pub fn log_or_unwrap<TValue, TError>(In(result): In<Result<TValue, TError>>) -> Option<TValue>
where
	Error: From<TError>,
{
	match result {
		Err(error) => {
			output(error);
			None
		}
		Ok(value) => Some(value),
	}
}

pub fn log_or_unwrap_option<TValue, TError>(
	In(result): In<Result<Option<TValue>, TError>>,
) -> Option<TValue>
where
	Error: From<TError>,
{
	match result {
		Err(error) => {
			output(error);
			None
		}
		Ok(Some(value)) => Some(value),
		Ok(None) => None,
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

pub mod test_tools {
	use super::*;
	use bevy::prelude::*;

	#[derive(Component, PartialEq, Debug)]
	pub struct FakeErrorLogMany(pub Vec<Error>);

	#[derive(Resource, PartialEq, Debug)]
	pub struct FakeErrorLogManyResource(pub Vec<Error>);

	#[derive(Component, Debug, PartialEq)]
	pub struct FakeErrorLog(pub Error);

	pub fn fake_log_error_lazy_many(
		entity: Entity,
	) -> impl FnMut(In<Vec<Result<(), Error>>>, Commands) {
		move |errors, mut commands| {
			let errors: Vec<_> = errors
				.0
				.iter()
				.filter_map(|result| result.clone().err())
				.collect();

			if errors.is_empty() {
				return;
			}

			let mut entity = commands.entity(entity);
			entity.insert(FakeErrorLogMany(errors));
		}
	}

	pub fn fake_log_error_many_recourse(
		result: In<Vec<Result<(), Error>>>,
		mut commands: Commands,
	) {
		let errors: Vec<_> = result
			.0
			.iter()
			.filter_map(|result| result.clone().err())
			.collect();

		if errors.is_empty() {
			return;
		}
		commands.insert_resource(FakeErrorLogManyResource(errors));
	}

	pub fn fake_log_error_lazy(agent: Entity) -> impl FnMut(In<Result<(), Error>>, Commands) {
		move |result, mut commands| {
			let Err(error) = result.0 else {
				return;
			};

			let mut agent = commands.entity(agent);
			agent.insert(FakeErrorLog(error));
		}
	}
}
