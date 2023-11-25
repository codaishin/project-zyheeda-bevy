use crate::errors::Error;
use bevy::ecs::system::In;
use tracing::error;

pub fn log(result: In<Result<(), Error>>) {
	let Err(error) = result.0 else {
		return;
	};

	error!("{:?}", error)
}

pub fn log_many(results: In<Vec<Result<(), Error>>>) {
	for error in results.0.iter().filter_map(|result| result.clone().err()) {
		error!("{:?}", error)
	}
}

#[cfg(test)]
pub mod tests {
	use super::*;
	use ::bevy::prelude::Entity;
	use bevy::ecs::{component::Component, system::Commands};

	#[derive(Component, PartialEq, Debug)]
	pub struct FakeErrorLogMany(pub Vec<Error>);

	#[derive(Component)]
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
