use bevy::{
	ecs::{
		component::Component,
		entity::Entity,
		system::{Commands, Query},
	},
	hierarchy::DespawnRecursiveExt,
};

pub(crate) const DESPAWN_DELAY: usize = 2;

#[derive(Component, Debug, PartialEq)]
pub(crate) struct DespawnAfterFrames(pub usize);

pub(crate) fn despawn_delayed(
	mut commands: Commands,
	mut despawns: Query<(Entity, &mut DespawnAfterFrames)>,
) {
	for (id, mut despawn) in &mut despawns {
		if despawn.0 > 0 {
			despawn.0 -= 1;
		} else {
			commands.entity(id).despawn_recursive();
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		hierarchy::BuildWorldChildren,
	};
	use common::test_tools::utils::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new_single_threaded([Update]);
		app.add_systems(Update, despawn_delayed);

		app
	}

	#[test]
	fn decrease_counter() {
		let mut app = setup();
		let despawn = app.world.spawn(DespawnAfterFrames(10)).id();

		app.update();

		let despawn = app.world.entity(despawn);

		assert_eq!(
			Some(&DespawnAfterFrames(9)),
			despawn.get::<DespawnAfterFrames>()
		);
	}

	#[test]
	fn despawn() {
		let mut app = setup();
		let despawn = app.world.spawn(DespawnAfterFrames(0)).id();
		let child = app.world.spawn_empty().set_parent(despawn).id();

		app.update();

		let despawn = app.world.get_entity(despawn);
		let child = app.world.get_entity(child);

		assert_eq!((true, true), (despawn.is_none(), child.is_none()));
	}
}
