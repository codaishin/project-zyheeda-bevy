use crate::components::Destroy;
use bevy::{
	ecs::{
		entity::Entity,
		query::With,
		system::{Commands, Query},
	},
	hierarchy::DespawnRecursiveExt,
};

pub(crate) fn destroy(mut commands: Commands, agents: Query<Entity, With<Destroy>>) {
	for agent in &agents {
		commands.entity(agent).despawn_recursive();
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
		let agent = app.world.spawn(Destroy).id();

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
		let agent = app.world.spawn(Destroy).id();
		let child = app.world.spawn_empty().set_parent(agent).id();

		app.update();

		assert!(!app.world.iter_entities().any(|e| e.id() == child));
	}
}
