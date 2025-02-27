use crate::errors::{Error, Level};
use bevy::ecs::system::In;
use tracing::{error, warn};

fn output(error: Error) {
	match error.lvl {
		Level::Error => error!("{}", error.msg),
		Level::Warning => warn!("{}", error.msg),
	}
}

pub fn log<TError>(result: In<Result<(), TError>>)
where
	Error: From<TError>,
{
	let Err(error) = result.0 else {
		return;
	};
	let error = Error::from(error);

	output(error);
}

pub fn log_many<TError>(results: In<Vec<Result<(), TError>>>)
where
	Error: From<TError>,
{
	for error in results.0.into_iter().filter_map(|result| result.err()) {
		let error = Error::from(error);
		output(error);
	}
}

pub mod test_tools {
	use super::*;
	use ::bevy::prelude::Entity;
	use bevy::ecs::{
		component::Component,
		system::{Commands, Resource},
	};

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
