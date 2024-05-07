use crate::components::SkillSpawn;
use bevy::{
	core::Name,
	ecs::{
		entity::Entity,
		system::{Commands, Query},
	},
};
use common::traits::{try_insert_on::TryInsertOn, try_remove_from::TryRemoveFrom};

pub(crate) fn add_skill_spawn(
	mut commands: Commands,
	agents: Query<(Entity, &SkillSpawn<&'static str>)>,
	names: Query<(Entity, &Name)>,
) {
	for (id, spawner) in &agents {
		let Some((spawner_id, ..)) = names.iter().find(|(.., n)| n.as_str() == spawner.0) else {
			continue;
		};
		commands.try_insert_on(id, SkillSpawn(spawner_id));
		commands.try_remove_from::<SkillSpawn<&'static str>>(id);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::SkillSpawn;
	use bevy::{
		app::{App, Update},
		core::Name,
	};
	use common::test_tools::utils::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, add_skill_spawn);

		app
	}

	#[test]
	fn set_entity() {
		let mut app = setup();
		let spawn = app.world.spawn(Name::from("spawner")).id();
		let agent = app.world.spawn(SkillSpawn("spawner")).id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(Some(&SkillSpawn(spawn)), agent.get::<SkillSpawn<Entity>>());
	}

	#[test]
	fn set_entity_only_once() {
		let mut app = setup();
		let spawn = app.world.spawn(Name::from("spawner")).id();
		let agent = app.world.spawn(SkillSpawn("spawner")).id();

		app.update();

		app.world.entity_mut(spawn).despawn();
		app.world.spawn(Name::from("spawner"));

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(Some(&SkillSpawn(spawn)), agent.get::<SkillSpawn<Entity>>());
	}
}
