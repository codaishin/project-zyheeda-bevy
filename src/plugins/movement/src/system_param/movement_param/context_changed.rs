use crate::system_param::movement_param::{MotionState, MovementContext, MovementParam};
use bevy::prelude::*;
use common::traits::accessors::get::ContextChanged;
use std::collections::HashSet;

impl<TMotion> ContextChanged for MovementContext<'_, TMotion>
where
	TMotion: Component,
{
	fn context_changed(&self) -> bool {
		let motion_changed = || match &self.motion {
			MotionState::Movement(movement) => movement.is_changed(),
			MotionState::JustRemoved => true,
			MotionState::Empty => false,
		};
		let speed_changed = || {
			self.current_speed
				.as_ref()
				.map(|s| s.is_changed())
				.unwrap_or(false)
		};

		motion_changed() || speed_changed()
	}
}

impl<TMotion> MovementParam<'_, '_, TMotion>
where
	TMotion: Component,
{
	pub(crate) fn update_just_removed(
		mut just_removed: ResMut<JustRemovedMovements>,
		mut removed: RemovedComponents<TMotion>,
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
	use crate::{components::config::SpeedIndex, system_param::movement_param::MovementParam};
	use common::traits::{accessors::get::GetContext, handles_movement::Movement};
	use testing::SingleThreadedApp;

	#[derive(Component)]
	struct _Motion;

	#[derive(Component, Debug, PartialEq)]
	struct _ContextChanged(bool);

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<JustRemovedMovements>().add_systems(
			Update,
			(
				MovementParam::<_Motion>::update_just_removed,
				|mut commands: Commands, m: MovementParam<_Motion>, entities: Query<Entity>| {
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
		let entity = app.world_mut().spawn(_Motion).id();

		app.update();

		assert_eq!(
			Some(&_ContextChanged(true)),
			app.world().entity(entity).get::<_ContextChanged>()
		);
	}

	#[test]
	fn is_changed_when_speed_index_added() {
		let mut app = setup();
		let entity = app.world_mut().spawn(SpeedIndex::default()).id();

		app.update();

		assert_eq!(
			Some(&_ContextChanged(true)),
			app.world().entity(entity).get::<_ContextChanged>()
		);
	}

	#[test]
	fn is_not_changed_when_movement_not_added() {
		let mut app = setup();
		let entity = app.world_mut().spawn(_Motion).id();

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
		let entity = app.world_mut().spawn(_Motion).id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<_Motion>()
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
		let entity = app.world_mut().spawn(_Motion).id();

		app.update();
		app.world_mut().entity_mut(entity).remove::<_Motion>();
		app.update();

		assert_eq!(
			Some(&_ContextChanged(true)),
			app.world().entity(entity).get::<_ContextChanged>()
		);
	}

	#[test]
	fn is_not_changed_when_movement_not_just_removed() {
		let mut app = setup();
		let entity = app.world_mut().spawn(_Motion).id();

		app.update();
		app.world_mut().entity_mut(entity).remove::<_Motion>();
		app.update();
		app.update();

		assert_eq!(
			Some(&_ContextChanged(false)),
			app.world().entity(entity).get::<_ContextChanged>()
		);
	}
}
