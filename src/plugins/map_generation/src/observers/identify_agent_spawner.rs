use crate::components::agent_spawner::AgentSpawner;
use bevy::{ecs::system::IntoObserverSystem, gltf::GltfMeshName, prelude::*};
use common::{
	traits::{accessors::get::TryApplyOn, handles_map_generation::AgentType},
	zyheeda_commands::ZyheedaCommands,
};
use std::collections::HashMap;

impl AgentSpawner {
	pub(crate) fn identify(
		agent_types: &[(&'static str, AgentType)],
	) -> impl IntoObserverSystem<Add, GltfMeshName, ()> {
		let agent_types = agent_types
			.iter()
			.copied()
			.collect::<HashMap<&'static str, AgentType>>();

		#[rustfmt::skip]
		let observer = move |
			added_name: On<Add, GltfMeshName>,
			names: Query<&GltfMeshName>,
			mut commands: ZyheedaCommands,
		| {
			let Ok(name) = names.get(added_name.entity) else {
				return;
			};
			let Some(agent_type) = agent_types.get(base(name)) else {
				return;
			};

			commands.try_apply_on(&added_name.entity, |mut e| {
				e.try_insert(AgentSpawner (*agent_type));
			});
		};

		IntoObserverSystem::into_system(observer)
	}
}

fn base(GltfMeshName(name): &GltfMeshName) -> &str {
	match name.find(".") {
		Some(dot) => &name[0..dot],
		None => name,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::traits::handles_enemies::EnemyType;
	use testing::SingleThreadedApp;

	macro_rules! gltf_mesh_name {
		($name:expr) => {
			GltfMeshName($name.to_owned())
		};
	}

	fn setup(config: &[(&'static str, AgentType)]) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(AgentSpawner::identify(config));

		app
	}

	#[test]
	fn insert_spawner() {
		let mut app = setup(&[
			("AA", AgentType::Player),
			("BB", AgentType::Enemy(EnemyType::VoidSphere)),
		]);

		let entities = [
			app.world_mut().spawn(gltf_mesh_name!("AA")).id(),
			app.world_mut().spawn(gltf_mesh_name!("BB")).id(),
		];

		assert_eq!(
			[
				Some(&AgentSpawner(AgentType::Player)),
				Some(&AgentSpawner(AgentType::Enemy(EnemyType::VoidSphere))),
			],
			app.world()
				.entity(entities)
				.map(|e| e.get::<AgentSpawner>())
		);
	}

	#[test]
	fn match_agent_to_name_until_dot() {
		let mut app = setup(&[
			("AA", AgentType::Player),
			("BB", AgentType::Enemy(EnemyType::VoidSphere)),
		]);

		let entities = [
			app.world_mut().spawn(gltf_mesh_name!("AA.12")).id(),
			app.world_mut().spawn(gltf_mesh_name!("BB.34")).id(),
		];

		assert_eq!(
			[
				Some(&AgentSpawner(AgentType::Player)),
				Some(&AgentSpawner(AgentType::Enemy(EnemyType::VoidSphere))),
			],
			app.world()
				.entity(entities)
				.map(|e| e.get::<AgentSpawner>())
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup(&[
			("AA", AgentType::Player),
			("BB", AgentType::Enemy(EnemyType::VoidSphere)),
		]);

		let mut entity = app.world_mut().spawn(gltf_mesh_name!("AA"));
		entity.insert(gltf_mesh_name!("BB"));

		assert_eq!(
			Some(&AgentSpawner(AgentType::Player)),
			entity.get::<AgentSpawner>(),
		);
	}
}
