use crate::components::ColliderRoot;
use bevy::{
	ecs::{
		entity::Entity,
		event::EventReader,
		system::{Commands, Query},
	},
	hierarchy::DespawnRecursiveExt,
};
use bevy_rapier3d::pipeline::CollisionEvent;

pub fn destroy_on_collision(
	mut commands: Commands,
	mut collision_events: EventReader<CollisionEvent>,
	roots: Query<&ColliderRoot>,
) {
	for collision_event in collision_events.read() {
		handle_collision(&mut commands, collision_event, &roots);
	}
}

fn handle_collision(
	commands: &mut Commands,
	collision_event: &CollisionEvent,
	roots: &Query<&ColliderRoot>,
) {
	let CollisionEvent::Started(a, b, _) = collision_event else {
		return;
	};
	despawn_from_root(commands, *a, roots);
	despawn_from_root(commands, *b, roots);
}

fn despawn_from_root(commands: &mut Commands, entity: Entity, roots: &Query<&ColliderRoot>) {
	let Ok(root) = roots.get(entity) else {
		return;
	};
	commands.entity(root.0).despawn_recursive();
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::ColliderRoot;
	use bevy::{
		app::{App, Update},
		hierarchy::BuildWorldChildren,
	};
	use bevy_rapier3d::rapier::geometry::CollisionEventFlags;

	fn setup() -> App {
		let mut app = App::new();
		app.add_event::<CollisionEvent>();
		app.add_systems(Update, destroy_on_collision);

		app
	}

	#[test]
	fn despawn() {
		let mut app = setup();

		let entity_a = app.world.spawn_empty().id();
		let entity_b = app.world.spawn_empty().id();
		let coll_a = app.world.spawn(ColliderRoot(entity_a)).id();
		let coll_b = app.world.spawn(ColliderRoot(entity_b)).id();

		app.world.send_event(CollisionEvent::Started(
			coll_a,
			coll_b,
			CollisionEventFlags::empty(),
		));

		app.update();

		assert_eq!(
			(None, None),
			(
				app.world.get_entity(entity_a).map(|e| e.id()),
				app.world.get_entity(entity_b).map(|e| e.id())
			)
		);
	}

	#[test]
	fn despawn_recursive() {
		let mut app = setup();

		let entity_a = app.world.spawn_empty().id();
		let entity_b = app.world.spawn_empty().id();
		let coll_a = app.world.spawn(ColliderRoot(entity_a)).id();
		let coll_b = app.world.spawn(ColliderRoot(entity_b)).id();
		app.world.entity_mut(entity_a).add_child(coll_a);
		app.world.entity_mut(entity_b).add_child(coll_b);

		app.world.send_event(CollisionEvent::Started(
			coll_a,
			coll_b,
			CollisionEventFlags::empty(),
		));

		app.update();

		assert!(app.world.entities().is_empty());
	}
}
