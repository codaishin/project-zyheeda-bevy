use bevy::ecs::{
	component::Component,
	entity::Entity,
	query::Added,
	system::{Commands, Query},
};
use bevy_rapier3d::geometry::{ActiveEvents, CollidingEntities, Sensor};
use common::traits::try_insert_on::TryInsertOn;

pub(crate) fn add_collider_config_components<TGravitySource: Component>(
	mut commands: Commands,
	sources: Query<Entity, Added<TGravitySource>>,
) {
	for source in &sources {
		commands.try_insert_on(
			source,
			(
				CollidingEntities::default(),
				Sensor,
				ActiveEvents::COLLISION_EVENTS,
			),
		);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::app::{App, Update};
	use bevy_rapier3d::geometry::{ActiveEvents, CollidingEntities, Sensor};
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Component)]
	struct _Source;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, add_collider_config_components::<_Source>);

		app
	}

	#[test]
	fn add_components_to_source() {
		let mut app = setup();

		let source = app.world_mut().spawn(_Source).id();

		app.update();

		let source = app.world().entity(source);

		assert_eq!(
			(true, true, Some(&ActiveEvents::COLLISION_EVENTS)),
			(
				source.contains::<CollidingEntities>(),
				source.contains::<Sensor>(),
				source.get::<ActiveEvents>(),
			)
		);
	}

	#[test]
	fn do_not_add_components_to_non_sources() {
		let mut app = setup();

		let source = app.world_mut().spawn_empty().id();

		app.update();

		let source = app.world().entity(source);

		assert_eq!(
			(false, false, None),
			(
				source.contains::<CollidingEntities>(),
				source.contains::<Sensor>(),
				source.get::<ActiveEvents>(),
			)
		);
	}

	#[test]
	fn add_components_only_when_first_adding_source() {
		let mut app = setup();

		let source = app.world_mut().spawn(_Source).id();

		app.update();

		app.world_mut()
			.entity_mut(source)
			.remove::<CollidingEntities>();
		app.world_mut().entity_mut(source).remove::<Sensor>();
		app.world_mut().entity_mut(source).remove::<ActiveEvents>();

		app.update();

		let source = app.world().entity(source);

		assert_eq!(
			(false, false, None),
			(
				source.contains::<CollidingEntities>(),
				source.contains::<Sensor>(),
				source.get::<ActiveEvents>(),
			)
		);
	}
}
