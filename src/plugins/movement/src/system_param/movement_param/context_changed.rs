use crate::{
	components::movement::path_or_direction::PathOrDirection,
	system_param::movement_param::{MovementContext, MovementParam},
};
use bevy::prelude::*;
use common::traits::accessors::get::ContextChanged;
use std::collections::HashSet;

impl ContextChanged for MovementContext<'_> {
	fn context_changed(&self) -> bool {
		match self {
			MovementContext::Movement(movement) => movement.is_changed(),
			MovementContext::JustRemoved => true,
			MovementContext::Empty => false,
		}
	}
}

impl MovementParam<'_, '_> {
	pub(crate) fn update_just_removed(
		mut just_removed: ResMut<JustRemovedMovements>,
		mut removed: RemovedComponents<PathOrDirection>,
	) {
		if !just_removed.0.is_empty() {
			just_removed.0.clear();
		}

		for entity in removed.read() {
			just_removed.0.insert(entity);
		}
	}
}

#[derive(Resource, Default)]
pub(crate) struct JustRemovedMovements(pub(crate) HashSet<Entity>);

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::movement::path_or_direction::PathOrDirection,
		system_param::movement_param::MovementParam,
	};
	use common::traits::{accessors::get::GetContext, handles_movement::Movement};
	use testing::SingleThreadedApp;

	struct _Motion;

	#[derive(Component, Debug, PartialEq)]
	struct _ContextChanged(bool);

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<JustRemovedMovements>().add_systems(
			Update,
			(
				MovementParam::update_just_removed,
				|mut commands: Commands, m: MovementParam, entities: Query<Entity>| {
					for entity in &entities {
						let key = Movement { entity };
						let Some(ctx) = MovementParam::get_context(&m, key) else {
							continue;
						};

						commands
							.entity(entity)
							.try_insert(_ContextChanged(ctx.context_changed()));
					}
				},
			)
				.chain(),
		);

		app
	}

	#[test]
	fn is_changed_when_movement_added() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(PathOrDirection::target(Vec3::new(1., 2., 3.)))
			.id();

		app.update();

		assert_eq!(
			Some(&_ContextChanged(true)),
			app.world().entity(entity).get::<_ContextChanged>()
		);
	}

	#[test]
	fn is_not_changed_when_movement_not_added() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(PathOrDirection::target(Vec3::new(1., 2., 3.)))
			.id();

		app.update();
		app.update();

		assert_eq!(
			Some(&_ContextChanged(false)),
			app.world().entity(entity).get::<_ContextChanged>()
		);
	}

	#[test]
	fn is_changed_when_movement_changed() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(PathOrDirection::target(Vec3::new(1., 2., 3.)))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<PathOrDirection>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some(&_ContextChanged(true)),
			app.world().entity(entity).get::<_ContextChanged>()
		);
	}

	#[test]
	fn is_changed_when_movement_just_removed() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(PathOrDirection::target(Vec3::new(1., 2., 3.)))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.remove::<PathOrDirection>();
		app.update();

		assert_eq!(
			Some(&_ContextChanged(true)),
			app.world().entity(entity).get::<_ContextChanged>()
		);
	}

	#[test]
	fn is_not_changed_when_movement_not_just_removed() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(PathOrDirection::target(Vec3::new(1., 2., 3.)))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.remove::<PathOrDirection>();
		app.update();
		app.update();

		assert_eq!(
			Some(&_ContextChanged(false)),
			app.world().entity(entity).get::<_ContextChanged>()
		);
	}
}
