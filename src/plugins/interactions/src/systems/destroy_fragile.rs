use crate::components::{Destroy, Fragile};
use bevy::ecs::{
	entity::Entity,
	event::EventReader,
	query::With,
	system::{Commands, Query},
};
use bevy_rapier3d::pipeline::CollisionEvent;
use common::components::ColliderRoot;

pub(crate) fn destroy_fragile(
	mut commands: Commands,
	mut collision_events: EventReader<CollisionEvent>,
	fragiles: Query<Entity, With<Fragile>>,
	roots: Query<&ColliderRoot>,
) {
	for event in collision_events.read() {
		let CollisionEvent::Started(a, b, ..) = event else {
			continue;
		};
		if let Ok(entity) = roots.get(*a).and_then(|r| fragiles.get(r.0)) {
			commands.entity(entity).insert(Destroy::Immediately);
		}
		if let Ok(entity) = roots.get(*b).and_then(|r| fragiles.get(r.0)) {
			commands.entity(entity).insert(Destroy::Immediately);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::Destroy;
	use bevy::app::{App, Update};
	use bevy_rapier3d::{pipeline::CollisionEvent, rapier::geometry::CollisionEventFlags};
	use common::{components::ColliderRoot, test_tools::utils::SingleThreadedApp};

	fn setup() -> App {
		let mut app = App::new_single_threaded([Update]);
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
}
