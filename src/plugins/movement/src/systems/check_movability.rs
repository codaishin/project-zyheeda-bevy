use crate::components::movable::Movable;
use bevy::{ecs::query::QueryFilter, prelude::*};
use common::errors::{ErrorData, Level};
use std::fmt::Display;

impl<T> CheckMovability for T where T: QueryFilter {}

pub(crate) trait CheckMovability: QueryFilter + Sized {
	fn check_movability(
		entities: Query<Entity, (Self, Without<Movable>)>,
	) -> Result<(), Vec<CannotMove>> {
		let errors = entities
			.iter()
			.map(|entity| CannotMove { entity })
			.collect::<Vec<_>>();

		if !errors.is_empty() {
			return Err(errors);
		}

		Ok(())
	}
}

#[derive(Debug, PartialEq)]
pub(crate) struct CannotMove {
	entity: Entity,
}

impl Display for CannotMove {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}: Cannot move. Did you register a movement definition via `HandlesMovement::register_movement`?",
			self.entity
		)
	}
}

impl ErrorData for CannotMove {
	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> impl Display {
		"Cannot Move"
	}

	fn into_details(self) -> impl Display {
		self
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use testing::SingleThreadedApp;

	#[derive(Component)]
	struct _Component;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn return_error_filter_matches() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn(_Component).id();

		let result = app
			.world_mut()
			.run_system_once(With::<_Component>::check_movability)?;

		assert_eq!(Err(vec![CannotMove { entity }]), result);
		Ok(())
	}

	#[test]
	fn return_ok_when_filter_matches_but_movable() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn((_Component, Movable));

		let result = app
			.world_mut()
			.run_system_once(With::<_Component>::check_movability)?;

		assert_eq!(Ok(()), result);
		Ok(())
	}

	#[test]
	fn return_ok_when_no_entity_matches_filter() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn_empty();

		let result = app
			.world_mut()
			.run_system_once(With::<_Component>::check_movability)?;

		assert_eq!(Ok(()), result);
		Ok(())
	}
}
