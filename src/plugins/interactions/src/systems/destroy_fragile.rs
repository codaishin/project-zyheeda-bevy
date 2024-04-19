use crate::components::{Destroy, Fragile};
use bevy::ecs::{
	entity::Entity,
	event::EventReader,
	query::With,
	system::{Commands, Query},
};
use bevy_rapier3d::{geometry::Sensor, pipeline::CollisionEvent};
use common::{components::ColliderRoot, traits::try_insert_on::TryInsertOn};

pub(crate) fn destroy_fragile(
	mut commands: Commands,
	mut collision_events: EventReader<CollisionEvent>,
	fragiles: Query<Entity, With<Fragile>>,
	roots: Query<&ColliderRoot>,
	sensors: Query<&Sensor>,
) {
	for event in collision_events.read() {
		let CollisionEvent::Started(a, b, ..) = event else {
			continue;
		};
		if let Some(entity) = valid_collision_entity(a, b, &fragiles, &roots, &sensors) {
			commands.try_insert_on(entity, Destroy::Immediately);
		}
		if let Some(entity) = valid_collision_entity(b, a, &fragiles, &roots, &sensors) {
			commands.try_insert_on(entity, Destroy::Immediately);
		}
	}
}

fn valid_collision_entity(
	entity: &Entity,
	other: &Entity,
	fragiles: &Query<Entity, With<Fragile>>,
	roots: &Query<&ColliderRoot>,
	sensors: &Query<&Sensor>,
) -> Option<Entity> {
	if sensors.contains(*other) {
		return None;
	};

	roots.get(*entity).and_then(|r| fragiles.get(r.0)).ok()
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::Destroy;
	use bevy::app::{App, Update};
	use bevy_rapier3d::{
		geometry::Sensor,
		pipeline::CollisionEvent,
		rapier::geometry::CollisionEventFlags,
	};
	use common::{components::ColliderRoot, test_tools::utils::SingleThreadedApp};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, destroy_fragile);
		app.add_event::<CollisionEvent>();

		app
	}

	#[test]
	fn destroy_on_collision() {
		let mut app = setup();

		let fragile = app.world.spawn(Fragile).id();
		let collider = app.world.spawn(ColliderRoot(fragile)).id();

		app.update();

		app.world.send_event(CollisionEvent::Started(
			collider,
			Entity::from_raw(42),
			CollisionEventFlags::empty(),
		));

		app.update();

		let fragile = app.world.entity(fragile);

		assert_eq!(Some(&Destroy::Immediately), fragile.get::<Destroy>());
	}

	#[test]
	fn do_not_destroy_on_collision_when_the_other_is_sensor() {
		let mut app = setup();

		let other = app.world.spawn(Sensor).id();
		let fragile = app.world.spawn(Fragile).id();
		let collider = app.world.spawn(ColliderRoot(fragile)).id();

		app.update();

		app.world.send_event(CollisionEvent::Started(
			collider,
			other,
			CollisionEventFlags::empty(),
		));

		app.update();

		let fragile = app.world.entity(fragile);

		assert_eq!(None, fragile.get::<Destroy>());
	}

	#[test]
	fn destroy_on_collision_reversed() {
		let mut app = setup();

		let fragile = app.world.spawn(Fragile).id();
		let collider = app.world.spawn(ColliderRoot(fragile)).id();

		app.update();

		app.world.send_event(CollisionEvent::Started(
			Entity::from_raw(42),
			collider,
			CollisionEventFlags::empty(),
		));

		app.update();

		let fragile = app.world.entity(fragile);

		assert_eq!(Some(&Destroy::Immediately), fragile.get::<Destroy>());
	}

	#[test]
	fn do_not_destroy_on_collision_when_the_other_is_sensor_reversed() {
		let mut app = setup();

		let other = app.world.spawn(Sensor).id();
		let fragile = app.world.spawn(Fragile).id();
		let collider = app.world.spawn(ColliderRoot(fragile)).id();

		app.update();

		app.world.send_event(CollisionEvent::Started(
			other,
			collider,
			CollisionEventFlags::empty(),
		));

		app.update();

		let fragile = app.world.entity(fragile);

		assert_eq!(None, fragile.get::<Destroy>());
	}
}
