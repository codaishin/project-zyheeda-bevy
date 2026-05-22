use crate::{GetNormalizedName, components::spawner::Spawner};
use bevy::{ecs::system::IntoObserverSystem, gltf::GltfMeshName, prelude::*};
use common::{
	traits::{
		accessors::get::TryApplyOn,
		handles_map_generation::PrefabType,
		thread_safe::ThreadSafe,
	},
	zyheeda_commands::ZyheedaCommands,
};
use std::collections::HashMap;
use zyheeda_core::strings::normalized_name::NormalizedName;

impl<T> Spawner<T>
where
	T: PrefabType + Copy + ThreadSafe,
{
	pub(crate) fn identify(
		types: &[(GetNormalizedName, T)],
	) -> impl IntoObserverSystem<Add, GltfMeshName, ()> {
		let types = types
			.iter()
			.map(|(n, i)| (n(), *i))
			.collect::<HashMap<_, _>>();

		#[rustfmt::skip]
		let observer = move |
			added_name: On<Add, GltfMeshName>,
			names: Query<&GltfMeshName>,
			mut commands: ZyheedaCommands,
		| {
			let Ok(GltfMeshName(name)) = names.get(added_name.entity) else {
				return;
			};
			let Some(entity_type) = types.get(&NormalizedName::from(name.as_str())) else {
				return;
			};

			commands.try_apply_on(&added_name.entity, |mut e| {
				e.try_insert((Spawner(*entity_type), Visibility::Hidden));
			});
		};

		IntoObserverSystem::into_system(observer)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::traits::{handles_enemies::EnemyType, handles_map_generation::AgentType};
	use testing::SingleThreadedApp;

	macro_rules! gltf_mesh_name {
		($name:expr) => {
			GltfMeshName($name.to_owned())
		};
	}

	fn setup(config: &[(GetNormalizedName, AgentType)]) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(Spawner::identify(config));

		app
	}

	#[test]
	fn insert_spawner() {
		let mut app = setup(&[
			(|| NormalizedName::from("AA"), AgentType::Player),
			(
				|| NormalizedName::from("BB"),
				AgentType::Enemy(EnemyType::VoidSphere),
			),
		]);

		let entities = [
			app.world_mut().spawn(gltf_mesh_name!("AA")).id(),
			app.world_mut().spawn(gltf_mesh_name!("BB")).id(),
		];

		assert_eq!(
			[
				Some(&Spawner(AgentType::Player)),
				Some(&Spawner(AgentType::Enemy(EnemyType::VoidSphere))),
			],
			app.world()
				.entity(entities)
				.map(|e| e.get::<Spawner<AgentType>>())
		);
	}

	#[test]
	fn insert_visibility_hidden() {
		let mut app = setup(&[
			(|| NormalizedName::from("AA"), AgentType::Player),
			(
				|| NormalizedName::from("BB"),
				AgentType::Enemy(EnemyType::VoidSphere),
			),
		]);

		let entities = [
			app.world_mut().spawn(gltf_mesh_name!("AA")).id(),
			app.world_mut().spawn(gltf_mesh_name!("BB")).id(),
		];

		assert_eq!(
			[Some(&Visibility::Hidden), Some(&Visibility::Hidden)],
			app.world().entity(entities).map(|e| e.get::<Visibility>()),
		);
	}

	#[test]
	fn match_agent_to_name_until_dot() {
		let mut app = setup(&[
			(|| NormalizedName::from("AA"), AgentType::Player),
			(
				|| NormalizedName::from("BB"),
				AgentType::Enemy(EnemyType::VoidSphere),
			),
		]);

		let entities = [
			app.world_mut().spawn(gltf_mesh_name!("AA.12")).id(),
			app.world_mut().spawn(gltf_mesh_name!("BB.34")).id(),
		];

		assert_eq!(
			[
				Some(&Spawner(AgentType::Player)),
				Some(&Spawner(AgentType::Enemy(EnemyType::VoidSphere))),
			],
			app.world()
				.entity(entities)
				.map(|e| e.get::<Spawner<AgentType>>())
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup(&[
			(|| NormalizedName::from("AA"), AgentType::Player),
			(
				|| NormalizedName::from("BB"),
				AgentType::Enemy(EnemyType::VoidSphere),
			),
		]);

		let mut entity = app.world_mut().spawn(gltf_mesh_name!("AA"));
		entity.insert(gltf_mesh_name!("BB"));

		assert_eq!(
			Some(&Spawner(AgentType::Player)),
			entity.get::<Spawner<AgentType>>(),
		);
	}
}
