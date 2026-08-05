use crate::{
	components::roles::RoleAssigned,
	materials::lit_material::LitType,
	resources::standard_materials::StandardMaterials,
};
use bevy::{gltf::GltfMaterialName, prelude::*};

impl StandardMaterials {
	const DEAD_SPACE: &str = "DeadSpace";

	pub(crate) fn set_lit_type(
		mut standard_materials: ResMut<Self>,
		meshes: Query<(Entity, Option<&GltfMaterialName>), Added<MeshMaterial3d<StandardMaterial>>>,
		agents: Query<(), With<RoleAssigned>>,
		parents: Query<&ChildOf>,
	) {
		let is_agent = |entity: Entity| agents.contains(entity);
		let parent_is_agent = |entity: Entity| parents.iter_ancestors(entity).any(is_agent);
		let is_dead_space = |name: &str| name == Self::DEAD_SPACE;

		for (entity, name) in meshes {
			let new_lit_type = match name {
				Some(GltfMaterialName(name)) if is_dead_space(name) => LitType::DeadSpace,
				_ if is_agent(entity) => LitType::Agent,
				_ if parent_is_agent(entity) => LitType::Agent,
				_ => continue,
			};

			let lit_type =
				standard_materials
					.entities
					.iter_mut()
					.find_map(|(_, (entities, lit_type))| {
						if !entities.contains(&entity) {
							return None;
						};

						Some(lit_type)
					});

			let Some(lit_type) = lit_type else {
				continue;
			};

			*lit_type = new_lit_type;
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::roles::RoleAssigned;
	use std::collections::{HashMap, HashSet};
	use testing::{SingleThreadedApp, new_handle};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, StandardMaterials::set_lit_type);

		app
	}

	#[test]
	fn set_dead_space() {
		let mut app = setup();
		let handle = new_handle();
		let entity = app
			.world_mut()
			.spawn((
				MeshMaterial3d::<StandardMaterial>(new_handle()),
				GltfMaterialName(String::from(StandardMaterials::DEAD_SPACE)),
			))
			.id();
		app.insert_resource(StandardMaterials {
			entities: HashMap::from([(handle.id(), (HashSet::from([entity]), LitType::Terrain))]),
		});

		app.update();

		assert_eq!(
			Some(LitType::DeadSpace),
			app.world()
				.resource::<StandardMaterials>()
				.entities
				.get(&handle.id())
				.map(|(_, lit_type)| *lit_type),
		);
	}

	#[test]
	fn do_not_set_dead_space() {
		let mut app = setup();
		let handle = new_handle();
		let entity = app
			.world_mut()
			.spawn((
				MeshMaterial3d::<StandardMaterial>(new_handle()),
				GltfMaterialName(format!("NOT_{}", StandardMaterials::DEAD_SPACE)),
			))
			.id();
		app.insert_resource(StandardMaterials {
			entities: HashMap::from([(handle.id(), (HashSet::from([entity]), LitType::Terrain))]),
		});

		app.update();

		assert_eq!(
			Some(LitType::Terrain),
			app.world()
				.resource::<StandardMaterials>()
				.entities
				.get(&handle.id())
				.map(|(_, lit_type)| *lit_type),
		);
	}

	#[test]
	fn set_agent() {
		let mut app = setup();
		let handle = new_handle();
		let entity = app
			.world_mut()
			.spawn((
				MeshMaterial3d::<StandardMaterial>(new_handle()),
				RoleAssigned,
			))
			.id();
		app.insert_resource(StandardMaterials {
			entities: HashMap::from([(handle.id(), (HashSet::from([entity]), LitType::Terrain))]),
		});

		app.update();

		assert_eq!(
			Some(LitType::Agent),
			app.world()
				.resource::<StandardMaterials>()
				.entities
				.get(&handle.id())
				.map(|(_, lit_type)| *lit_type),
		);
	}

	#[test]
	fn set_agent_from_distant_ancestor() {
		let mut app = setup();
		let handle = new_handle();
		let ancestor = app.world_mut().spawn(RoleAssigned).id();
		let intermediate = app.world_mut().spawn(ChildOf(ancestor)).id();
		let entity = app
			.world_mut()
			.spawn((
				MeshMaterial3d::<StandardMaterial>(new_handle()),
				ChildOf(intermediate),
			))
			.id();
		app.insert_resource(StandardMaterials {
			entities: HashMap::from([(handle.id(), (HashSet::from([entity]), LitType::Terrain))]),
		});

		app.update();

		assert_eq!(
			Some(LitType::Agent),
			app.world()
				.resource::<StandardMaterials>()
				.entities
				.get(&handle.id())
				.map(|(_, lit_type)| *lit_type),
		);
	}

	#[test]
	fn prefer_dead_space_over_agent() {
		let mut app = setup();
		let handle = new_handle();
		let entity = app
			.world_mut()
			.spawn((
				MeshMaterial3d::<StandardMaterial>(new_handle()),
				GltfMaterialName(String::from(StandardMaterials::DEAD_SPACE)),
				RoleAssigned,
			))
			.id();
		app.insert_resource(StandardMaterials {
			entities: HashMap::from([(handle.id(), (HashSet::from([entity]), LitType::Terrain))]),
		});

		app.update();

		assert_eq!(
			Some(LitType::DeadSpace),
			app.world()
				.resource::<StandardMaterials>()
				.entities
				.get(&handle.id())
				.map(|(_, lit_type)| *lit_type),
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		let handle = new_handle();
		let entity = app
			.world_mut()
			.spawn((
				MeshMaterial3d::<StandardMaterial>(new_handle()),
				GltfMaterialName(String::from(StandardMaterials::DEAD_SPACE)),
			))
			.id();
		app.insert_resource(StandardMaterials {
			entities: HashMap::from([(handle.id(), (HashSet::from([entity]), LitType::Terrain))]),
		});

		app.update();
		for (_, lit_type) in app
			.world_mut()
			.resource_mut::<StandardMaterials>()
			.entities
			.values_mut()
		{
			*lit_type = LitType::Terrain;
		}
		app.update();

		assert_eq!(
			Some(LitType::Terrain),
			app.world()
				.resource::<StandardMaterials>()
				.entities
				.get(&handle.id())
				.map(|(_, dead_space)| *dead_space),
		);
	}

	#[test]
	fn act_again_if_new_matching_entity_spawned() {
		let mut app = setup();
		let handle = new_handle();
		let entity = app.world_mut().spawn_empty().id();
		app.insert_resource(StandardMaterials {
			entities: HashMap::from([(handle.id(), (HashSet::from([entity]), LitType::Terrain))]),
		});

		app.update();
		app.world_mut().entity_mut(entity).insert((
			MeshMaterial3d::<StandardMaterial>(new_handle()),
			GltfMaterialName(String::from(StandardMaterials::DEAD_SPACE)),
		));
		app.update();

		assert_eq!(
			Some(LitType::DeadSpace),
			app.world()
				.resource::<StandardMaterials>()
				.entities
				.get(&handle.id())
				.map(|(_, dead_space)| *dead_space),
		);
	}
}
