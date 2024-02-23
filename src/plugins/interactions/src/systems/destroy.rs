use crate::components::Destroy;
use bevy::{
	ecs::{
		entity::Entity,
		system::{Commands, Query},
	},
	hierarchy::DespawnRecursiveExt,
};

pub(crate) fn destroy(mut commands: Commands, mut agents: Query<(Entity, &mut Destroy)>) {
	for (id, mut destroy) in &mut agents {
		match *destroy {
			Destroy::Immediately | Destroy::AfterFrames(0) => {
				commands.entity(id).despawn_recursive()
			}
			Destroy::AfterFrames(count) => *destroy = Destroy::AfterFrames(count - 1),
		};
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::Destroy;
	use bevy::{
		app::{App, Update},
		hierarchy::BuildWorldChildren,
	};

	fn setup() -> App {
		let mut app = App::new();
		app.add_systems(Update, destroy);

		app
	}

	#[test]
	fn despawn_when_destroy_component_attached() {
		let mut app = setup();
		let agent = app.world.spawn(Destroy::Immediately).id();

		app.update();

		assert!(!app.world.iter_entities().any(|e| e.id() == agent));
	}

	#[test]
	fn do_not_despawn_when_destroy_component_not_attached() {
		let mut app = setup();
		let agent = app.world.spawn_empty().id();

		app.update();

		assert!(app.world.iter_entities().any(|e| e.id() == agent));
	}

	#[test]
	fn despawn_recursive_when_destroy_component_attached() {
		let mut app = setup();
		let agent = app.world.spawn(Destroy::Immediately).id();
		let child = app.world.spawn_empty().set_parent(agent).id();

		app.update();

		assert!(!app.world.iter_entities().any(|e| e.id() == child));
	}

	#[test]
	fn decrease_delay_counter() {
		let mut app = setup();
		let despawn = app.world.spawn(Destroy::AfterFrames(10)).id();

		app.update();

		let despawn = app.world.entity(despawn);

		assert_eq!(Some(&Destroy::AfterFrames(9)), despawn.get::<Destroy>());
	}

	#[test]
	fn despawn() {
		let mut app = setup();
		let despawn = app.world.spawn(Destroy::AfterFrames(0)).id();
		let child = app.world.spawn_empty().set_parent(despawn).id();

		app.update();

		let despawn = app.world.get_entity(despawn);
		let child = app.world.get_entity(child);

		assert_eq!((true, true), (despawn.is_none(), child.is_none()));
	}
}
