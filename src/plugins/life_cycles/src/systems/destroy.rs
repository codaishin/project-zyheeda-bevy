use crate::components::destroy::Destroy;
use bevy::prelude::*;
use common::traits::try_despawn::TryDespawn;

pub(crate) fn destroy(mut commands: Commands, mut agents: Query<Entity, With<Destroy>>) {
	for entity in &mut agents {
		commands.try_despawn(entity);
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn setup() -> App {
		let mut app = App::new();
		app.add_systems(Update, destroy);

		app
	}

	#[test]
	fn despawn_when_destroy_component_attached() {
		let mut app = setup();
		let agent = app.world_mut().spawn(Destroy).id();

		app.update();

		assert!(!app.world().iter_entities().any(|e| e.id() == agent));
	}

	#[test]
	fn do_not_despawn_when_destroy_component_not_attached() {
		let mut app = setup();
		let agent = app.world_mut().spawn_empty().id();

		app.update();

		assert!(app.world().iter_entities().any(|e| e.id() == agent));
	}

	#[test]
	fn despawn_recursive_when_destroy_component_attached() {
		let mut app = setup();
		let agent = app.world_mut().spawn(Destroy).id();
		let child = app.world_mut().spawn(ChildOf(agent)).id();

		app.update();

		assert!(!app.world().iter_entities().any(|e| e.id() == child));
	}
}
