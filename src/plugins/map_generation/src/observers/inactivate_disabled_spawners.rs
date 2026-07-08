use crate::components::{
	map::{Map, objects::MapObjectOf},
	spawner_active::SpawnerActive,
};
use bevy::{gltf::GltfMeshName, prelude::*};
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

impl SpawnerActive {
	pub(crate) fn remove_from_disabled_sources(
		on_add: On<Add, MapObjectOf>,
		mut commands: ZyheedaCommands,
		objects: Query<(&MapObjectOf, &GltfMeshName), With<Self>>,
		maps: Query<&Map>,
	) {
		let Ok((MapObjectOf(map), GltfMeshName(name))) = objects.get(on_add.entity) else {
			return;
		};
		let Ok(map) = maps.get(*map) else {
			return;
		};

		if !map.disabled_object_sources.contains(name) {
			return;
		}

		commands.try_apply_on(&on_add.entity, |mut e| {
			e.try_remove::<SpawnerActive>();
		});
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::map::MapObjectSource;
	use test_case::test_case;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(SpawnerActive::remove_from_disabled_sources);

		app
	}

	fn map_with_persistent<const N: usize>(persistent: [&str; N]) -> Map {
		Map {
			disabled_object_sources: persistent
				.into_iter()
				.map(str::to_owned)
				.map(MapObjectSource)
				.collect(),
		}
	}

	#[test_case(map_with_persistent(["persistent"]), false; "spawned is persistent")]
	#[test_case(map_with_persistent([]), true; "spawned is not persistent")]
	fn control_active_marker(map: Map, is_active: bool) {
		let mut app = setup();
		let map = app.world_mut().spawn(map).id();

		let mut spawner = app
			.world_mut()
			.spawn((SpawnerActive, GltfMeshName(String::from("persistent"))));
		spawner.insert(MapObjectOf(map));

		assert_eq!(is_active, spawner.contains::<SpawnerActive>());
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		let map = app
			.world_mut()
			.spawn(map_with_persistent(["persistent"]))
			.id();

		let mut spawner = app
			.world_mut()
			.spawn((SpawnerActive, GltfMeshName(String::from("persistent"))));
		spawner.insert(MapObjectOf(map));
		spawner.insert(SpawnerActive);
		spawner.insert(MapObjectOf(map));

		assert!(spawner.contains::<SpawnerActive>());
	}
}
