use bevy::ecs::{
	component::Component,
	entity::Entity,
	query::Added,
	system::{Commands, Query},
};
use bevy_rapier3d::geometry::CollidingEntities;
use common::traits::try_insert_on::TryInsertOn;

pub(crate) fn add_colliding_entities<TGravitySource: Component>(
	mut commands: Commands,
	sources: Query<Entity, Added<TGravitySource>>,
) {
	for source in &sources {
		commands.try_insert_on(source, CollidingEntities::default());
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::app::{App, Update};
	use bevy_rapier3d::geometry::CollidingEntities;
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Component)]
	struct _Source;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, add_colliding_entities::<_Source>);

		app
	}

	#[test]
	fn add_colliding_entities_to_source() {
		let mut app = setup();

		let source = app.world.spawn(_Source).id();

		app.update();

		let source = app.world.entity(source);

		assert!(source.contains::<CollidingEntities>());
	}

	#[test]
	fn do_not_add_colliding_entities_to_non_sources() {
		let mut app = setup();

		let source = app.world.spawn_empty().id();

		app.update();

		let source = app.world.entity(source);

		assert!(!source.contains::<CollidingEntities>());
	}

	#[test]
	fn do_not_add_colliding_entities_only_when_first_adding_source() {
		let mut app = setup();

		let source = app.world.spawn(_Source).id();

		app.update();

		app.world.entity_mut(source).remove::<CollidingEntities>();

		app.update();

		let source = app.world.entity(source);

		assert!(!source.contains::<CollidingEntities>());
	}
}
