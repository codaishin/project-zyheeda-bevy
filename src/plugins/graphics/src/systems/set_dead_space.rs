use crate::{materials::lit_material::DeadSpace, resources::standard_materials::StandardMaterials};
use bevy::{gltf::GltfMaterialName, prelude::*};

impl StandardMaterials {
	pub(crate) fn set_dead_space_for_material(
		dead_space_name: &'static str,
	) -> impl IntoSystem<(), (), ()> {
		#[rustfmt::skip]
		let system = move |
			mut standard_materials: ResMut<Self>,
			material_names: Query<(Entity, &GltfMaterialName), Added<GltfMaterialName>>
		| {
			for (entity, GltfMaterialName(name)) in material_names {
				if name != dead_space_name {
					continue;
				}

				let dead_space = standard_materials.entities.iter_mut().find_map(
					|(_, (entities, dead_space))| {
						if !entities.contains(&entity) {
							return None;
						};

						Some(dead_space)
					},
				);

				let Some(DeadSpace(dead_space)) = dead_space else {
					continue;
				};

				*dead_space = true;
			}
		};

		IntoSystem::into_system(system)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::materials::lit_material::DeadSpace;
	use std::collections::{HashMap, HashSet};
	use testing::{SingleThreadedApp, new_handle};

	fn setup(name: &'static str) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, StandardMaterials::set_dead_space_for_material(name));

		app
	}

	#[test]
	fn set_dead_space() {
		let mut app = setup("my name");
		let handle = new_handle();
		let entity = app
			.world_mut()
			.spawn(GltfMaterialName(String::from("my name")))
			.id();
		app.insert_resource(StandardMaterials {
			entities: HashMap::from([(handle.id(), (HashSet::from([entity]), DeadSpace(false)))]),
		});

		app.update();

		assert_eq!(
			Some(DeadSpace(true)),
			app.world()
				.resource::<StandardMaterials>()
				.entities
				.get(&handle.id())
				.map(|(_, dead_space)| *dead_space),
		);
	}

	#[test]
	fn do_not_set_dead_space() {
		let mut app = setup("my name");
		let handle = new_handle();
		let entity = app
			.world_mut()
			.spawn(GltfMaterialName(String::from("my other name")))
			.id();
		app.insert_resource(StandardMaterials {
			entities: HashMap::from([(handle.id(), (HashSet::from([entity]), DeadSpace(false)))]),
		});

		app.update();

		assert_eq!(
			Some(DeadSpace(false)),
			app.world()
				.resource::<StandardMaterials>()
				.entities
				.get(&handle.id())
				.map(|(_, dead_space)| *dead_space),
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup("my name");
		let handle = new_handle();
		let entity = app
			.world_mut()
			.spawn(GltfMaterialName(String::from("my name")))
			.id();
		app.insert_resource(StandardMaterials {
			entities: HashMap::from([(handle.id(), (HashSet::from([entity]), DeadSpace(false)))]),
		});

		app.update();
		for (_, DeadSpace(dead_space)) in app
			.world_mut()
			.resource_mut::<StandardMaterials>()
			.entities
			.values_mut()
		{
			*dead_space = false;
		}
		app.update();

		assert_eq!(
			Some(DeadSpace(false)),
			app.world()
				.resource::<StandardMaterials>()
				.entities
				.get(&handle.id())
				.map(|(_, dead_space)| *dead_space),
		);
	}

	#[test]
	fn act_again_if_new_matching_entity_spawned() {
		let mut app = setup("my name");
		let handle = new_handle();
		let entity = app.world_mut().spawn_empty().id();
		app.insert_resource(StandardMaterials {
			entities: HashMap::from([(handle.id(), (HashSet::from([entity]), DeadSpace(false)))]),
		});

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(GltfMaterialName(String::from("my name")));
		app.update();

		assert_eq!(
			Some(DeadSpace(true)),
			app.world()
				.resource::<StandardMaterials>()
				.entities
				.get(&handle.id())
				.map(|(_, dead_space)| *dead_space),
		);
	}
}
