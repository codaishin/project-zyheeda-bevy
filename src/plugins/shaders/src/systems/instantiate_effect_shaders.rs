use crate::{
	components::effect_shader::{EffectShaders, MeshData},
	traits::insert_unmovable_effect_shader::InsertUnmovableEffectShader,
};
use bevy::prelude::*;
use common::components::Unmovable;

pub(crate) fn instantiate_effect_shaders(
	mut commands: Commands,
	effect_shaders: Query<&EffectShaders, Changed<EffectShaders>>,
	effect_sub_shader: Query<&EffectSubShader>,
	children: Query<&Children>,
) {
	for shaders in &effect_shaders {
		clear(&mut commands, shaders, &effect_sub_shader, &children);
		instantiate(&mut commands, shaders);
	}
}

#[derive(Component)]
pub(crate) struct EffectSubShader;

fn clear(
	commands: &mut Commands,
	effect_shaders: &EffectShaders,
	effect_sub_shader: &Query<&EffectSubShader>,
	children: &Query<&Children>,
) {
	let is_effect_sub_shader = |child: &&Entity| effect_sub_shader.contains(**child);

	for MeshData { entity, .. } in &effect_shaders.meshes {
		let Ok(children) = children.get(*entity) else {
			continue;
		};

		for sub_shader in children.iter().filter(is_effect_sub_shader) {
			let Some(sub_shader) = commands.get_entity(*sub_shader) else {
				continue;
			};

			sub_shader.despawn_recursive();
		}
	}
}

fn instantiate(commands: &mut Commands, shaders: &EffectShaders) {
	for shader in &shaders.shaders {
		for MeshData { handle, entity } in &shaders.meshes {
			let mut sub_shader = commands.spawn(EffectSubShader);
			sub_shader.set_parent(*entity);

			sub_shader.insert((
				handle.clone(),
				Unmovable::<Handle<Mesh>>::default(),
				SpatialBundle::default(),
			));
			sub_shader.insert_unmovable_effect_shader(shader);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::effect_shader::{EffectShader, EffectShaders, MeshData};
	use common::{
		assert_bundle,
		components::Unmovable,
		test_tools::utils::{new_handle, SingleThreadedApp},
	};
	use std::collections::HashSet;

	#[derive(Asset, TypePath)]
	struct _Shader1;

	#[derive(Asset, TypePath)]
	struct _Shader2;

	fn child_of(entity: Entity) -> impl FnMut(&EntityRef<'_>) -> bool {
		move |child| {
			child
				.get::<Parent>()
				.map(|parent| parent.get() == entity)
				.unwrap_or(false)
		}
	}

	fn find_children(app: &App, entity: Entity) -> impl Iterator<Item = EntityRef> {
		app.world().iter_entities().filter(child_of(entity))
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, instantiate_effect_shaders);

		app
	}

	#[test]
	fn insert_single_shader_effect() {
		let mut app = setup();
		let mesh_entity = app.world_mut().spawn_empty().id();
		let handle = new_handle::<_Shader1>();
		let shader = EffectShader::from(handle.clone());
		let shaders = EffectShaders {
			meshes: vec![MeshData {
				entity: mesh_entity,
				handle: new_handle(),
			}],
			shaders: vec![shader],
		};
		app.world_mut().spawn(shaders);

		app.update();

		assert_eq!(
			vec![(
				Some(&handle),
				Some(&Unmovable::<Handle<_Shader1>>::default())
			)],
			find_children(&app, mesh_entity)
				.map(|child| (
					child.get::<Handle<_Shader1>>(),
					child.get::<Unmovable<Handle<_Shader1>>>()
				))
				.collect::<Vec<_>>()
		)
	}

	#[test]
	fn insert_single_mesh_copy() {
		let mut app = setup();
		let mesh_entity = app.world_mut().spawn_empty().id();
		let handle = new_handle::<Mesh>();
		let shader = EffectShader::from(new_handle::<_Shader1>());
		let shaders = EffectShaders {
			meshes: vec![MeshData {
				entity: mesh_entity,
				handle: handle.clone(),
			}],
			shaders: vec![shader],
		};
		app.world_mut().spawn(shaders);

		app.update();

		assert_eq!(
			vec![(Some(&handle), Some(&Unmovable::<Handle<Mesh>>::default()))],
			find_children(&app, mesh_entity)
				.map(|child| (
					child.get::<Handle<Mesh>>(),
					child.get::<Unmovable<Handle<Mesh>>>()
				))
				.collect::<Vec<_>>()
		)
	}

	#[test]
	fn insert_spatial_bundle() {
		let mut app = setup();
		let mesh_entity = app.world_mut().spawn_empty().id();
		let mesh = new_handle::<Mesh>();
		let shader = EffectShader::from(new_handle::<_Shader1>());
		let shaders = EffectShaders {
			meshes: vec![MeshData {
				entity: mesh_entity,
				handle: mesh,
			}],
			shaders: vec![shader],
		};
		app.world_mut().spawn(shaders);

		app.update();

		let child = find_children(&app, mesh_entity).next().unwrap();
		assert_bundle!(SpatialBundle, &app, child);
	}

	#[test]
	fn pair_each_mesh_with_one_shader() {
		let mut app = setup();
		let meshes = vec![
			MeshData {
				entity: app.world_mut().spawn_empty().id(),
				handle: new_handle(),
			},
			MeshData {
				entity: app.world_mut().spawn_empty().id(),
				handle: new_handle(),
			},
		];
		let shader1 = new_handle::<_Shader1>();
		let shader2 = new_handle::<_Shader2>();
		let shaders = EffectShaders {
			meshes: meshes.clone(),
			shaders: vec![
				EffectShader::from(shader1.clone()),
				EffectShader::from(shader2.clone()),
			],
		};
		app.world_mut().spawn(shaders);

		app.update();

		assert_eq!(
			HashSet::from([
				(Some(&meshes[0].handle), Some(&shader1), None),
				(Some(&meshes[0].handle), None, Some(&shader2)),
				(Some(&meshes[1].handle), Some(&shader1), None),
				(Some(&meshes[1].handle), None, Some(&shader2)),
			]),
			find_children(&app, meshes[0].entity)
				.chain(find_children(&app, meshes[1].entity))
				.map(|entity| (
					entity.get::<Handle<Mesh>>(),
					entity.get::<Handle<_Shader1>>(),
					entity.get::<Handle<_Shader2>>(),
				))
				.collect::<HashSet<_>>()
		)
	}

	#[test]
	fn do_not_spawn_children_twice() {
		let mut app = setup();
		let mesh_entity = app.world_mut().spawn_empty().id();
		let shaders = EffectShaders {
			meshes: vec![MeshData {
				entity: mesh_entity,
				handle: new_handle(),
			}],
			shaders: vec![EffectShader::from(new_handle::<_Shader1>())],
		};
		app.world_mut().spawn(shaders);

		app.update();
		app.update();

		assert_eq!(
			vec![(true, true)],
			find_children(&app, mesh_entity)
				.map(|child| (
					child.contains::<Handle<Mesh>>(),
					child.contains::<Handle<_Shader1>>(),
				))
				.collect::<Vec<_>>()
		)
	}

	#[test]
	fn rewrite_again_when_effect_shaders_mutably_dereferenced() {
		let mut app = setup();
		let mesh_entity = app.world_mut().spawn_empty().id();
		let shaders = EffectShaders {
			meshes: vec![MeshData {
				entity: mesh_entity,
				handle: new_handle(),
			}],
			shaders: vec![EffectShader::from(new_handle::<_Shader1>())],
		};
		let entity = app.world_mut().spawn(shaders).id();

		app.update();

		app.world_mut()
			.entity_mut(entity)
			.get_mut::<EffectShaders>()
			.unwrap()
			.shaders = vec![EffectShader::from(new_handle::<_Shader2>())];

		app.update();

		assert_eq!(
			vec![(true, false, true)],
			find_children(&app, mesh_entity)
				.map(|child| (
					child.contains::<Handle<Mesh>>(),
					child.contains::<Handle<_Shader1>>(),
					child.contains::<Handle<_Shader2>>(),
				))
				.collect::<Vec<_>>()
		);
	}

	#[test]
	fn when_rewriting_despawn_children_recursively() {
		#[derive(Component, Debug, PartialEq)]
		struct _DeepChild;

		let mut app = setup();
		let mesh_entity = app.world_mut().spawn_empty().id();
		let shaders = EffectShaders {
			meshes: vec![MeshData {
				entity: mesh_entity,
				handle: new_handle(),
			}],
			shaders: vec![EffectShader::from(new_handle::<_Shader1>())],
		};
		let entity = app.world_mut().spawn(shaders).id();

		app.update();

		app.world_mut()
			.entity_mut(entity)
			.get_mut::<EffectShaders>()
			.unwrap()
			.shaders = vec![EffectShader::from(new_handle::<_Shader2>())];
		let mesh_entity_child = find_children(&app, mesh_entity)
			.map(|e| e.id())
			.next()
			.unwrap();
		app.world_mut()
			.spawn(_DeepChild)
			.set_parent(mesh_entity_child);

		app.update();

		assert_eq!(
			vec![] as Vec<&_DeepChild>,
			app.world()
				.iter_entities()
				.filter_map(|e| e.get::<_DeepChild>())
				.collect::<Vec<_>>()
		)
	}

	#[test]
	fn when_rewriting_do_not_despawn_children_that_were_not_spawned_by_system() {
		let mut app = setup();
		let mesh_entity = app.world_mut().spawn_empty().id();
		let shaders = EffectShaders {
			meshes: vec![MeshData {
				entity: mesh_entity,
				handle: new_handle(),
			}],
			shaders: vec![EffectShader::from(new_handle::<_Shader1>())],
		};
		let entity = app.world_mut().spawn(shaders).id();

		app.update();

		app.world_mut()
			.entity_mut(entity)
			.get_mut::<EffectShaders>()
			.unwrap()
			.shaders = vec![EffectShader::from(new_handle::<_Shader2>())];
		let child = app.world_mut().spawn_empty().set_parent(mesh_entity).id();

		app.update();

		assert!(app.world().get_entity(child).is_some());
	}
}
