use crate::{GetNormalizedName, components::interactive_spawner::InteractiveSpawner};
use bevy::{ecs::system::IntoObserverSystem, gltf::GltfMeshName, prelude::*};
use common::{
	traits::{accessors::get::TryApplyOn, handles_interactive::Interactive},
	zyheeda_commands::ZyheedaCommands,
};
use std::collections::HashMap;
use zyheeda_core::strings::normalized_name::NormalizedName;

impl InteractiveSpawner {
	pub(crate) fn identify(
		agent_types: &[(GetNormalizedName, Interactive)],
	) -> impl IntoObserverSystem<Add, GltfMeshName, ()> {
		let interactive_types = agent_types
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
			let Some(interactive_type) = interactive_types.get(&NormalizedName::from(name.as_str())) else {
				return;
			};

			commands.try_apply_on(&added_name.entity, |mut e| {
				e.try_insert((InteractiveSpawner(*interactive_type), Visibility::Hidden));
			});
		};

		IntoObserverSystem::into_system(observer)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::traits::handles_interactive::Door;
	use testing::SingleThreadedApp;

	macro_rules! gltf_mesh_name {
		($name:expr) => {
			GltfMeshName($name.to_owned())
		};
	}

	fn setup(config: &[(GetNormalizedName, Interactive)]) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(InteractiveSpawner::identify(config));

		app
	}

	#[test]
	fn insert_spawner() {
		let mut app = setup(&[(
			|| NormalizedName::from("AA"),
			Interactive::Door(Door::SlideDoor),
		)]);

		let entities = [app.world_mut().spawn(gltf_mesh_name!("AA")).id()];

		assert_eq!(
			[Some(&InteractiveSpawner(Interactive::Door(
				Door::SlideDoor
			)))],
			app.world()
				.entity(entities)
				.map(|e| e.get::<InteractiveSpawner>())
		);
	}

	#[test]
	fn insert_visibility_hidden() {
		let mut app = setup(&[(
			|| NormalizedName::from("AA"),
			Interactive::Door(Door::SlideDoor),
		)]);

		let entities = [app.world_mut().spawn(gltf_mesh_name!("AA")).id()];

		assert_eq!(
			[Some(&Visibility::Hidden)],
			app.world().entity(entities).map(|e| e.get::<Visibility>()),
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup(&[(
			|| NormalizedName::from("AA"),
			Interactive::Door(Door::SlideDoor),
		)]);

		let mut entity = app.world_mut().spawn(gltf_mesh_name!("AA"));
		entity.remove::<InteractiveSpawner>();
		entity.insert(gltf_mesh_name!("BB"));

		assert_eq!(None, entity.get::<InteractiveSpawner>());
	}
}
