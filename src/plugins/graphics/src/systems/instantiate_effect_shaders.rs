use crate::{
	components::effect_shaders_target::{EffectShaderHandle, EffectShaderMeshes, EffectShaders},
	traits::{
		insert_protected_effect_shader::InsertProtectedEffectShader,
		remove_protected_effect_shader::RemoveProtectedEffectShader,
	},
};
use bevy::prelude::*;
use common::{
	traits::accessors::get::{GetMut, TryApplyOn},
	zyheeda_commands::ZyheedaCommands,
};
use std::collections::HashSet;

#[allow(clippy::type_complexity)]
pub(crate) fn instantiate_effect_shaders(
	mut commands: ZyheedaCommands,
	effect_shaders: Query<
		(Entity, &EffectShaders, &EffectShaderMeshes, Option<&Active>),
		Or<(Changed<EffectShaders>, Changed<EffectShaderMeshes>)>,
	>,
) {
	for (entity, effect_shaders, meshes, active) in &effect_shaders {
		clear(&mut commands, meshes, active);
		instantiate(&mut commands, effect_shaders, meshes);
		commands.try_apply_on(&entity, |mut e| {
			e.try_insert(Active(effect_shaders.shaders.clone()));
		});
	}
}

#[derive(Component)]
pub(crate) struct Active(HashSet<EffectShaderHandle>);

fn clear(
	commands: &mut ZyheedaCommands,
	effect_shaders: &EffectShaderMeshes,
	active: Option<&Active>,
) {
	let Some(Active(shaders)) = active else {
		return;
	};

	for shader in shaders {
		for entity in effect_shaders.iter() {
			let Some(mut entity) = commands.get_mut(&entity) else {
				continue;
			};

			entity.remove_protected_effect_shader(shader);
		}
	}
}

fn instantiate(
	commands: &mut ZyheedaCommands,
	effect_shaders: &EffectShaders,
	meshes: &EffectShaderMeshes,
) {
	for shader in &effect_shaders.shaders {
		for entity in meshes.iter() {
			let Some(mut entity) = commands.get_mut(&entity) else {
				continue;
			};

			entity.insert_protected_effect_shader(shader);
		}
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::components::effect_shaders_target::{
		EffectShaderHandle,
		EffectShaderMeshOf,
		EffectShaders,
	};
	use bevy::render::render_resource::AsBindGroup;
	use common::components::protected::Protected;
	use std::collections::HashSet;
	use testing::{SingleThreadedApp, new_handle};

	#[derive(Asset, TypePath, Clone, AsBindGroup, PartialEq, Eq, Hash, Debug)]
	struct _Shader1 {}

	impl Material for _Shader1 {}

	#[derive(Asset, TypePath, Clone, AsBindGroup, PartialEq, Eq, Hash, Debug)]
	struct _Shader2 {}

	impl Material for _Shader2 {}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, instantiate_effect_shaders);

		app
	}

	#[test]
	fn insert_single_shader_effect() {
		let mut app = setup();
		let handle = new_handle::<_Shader1>();
		let shader = EffectShaderHandle::from(handle.clone());
		let shaders = EffectShaders {
			shaders: HashSet::from([shader]),
		};
		let root = app.world_mut().spawn(shaders).id();
		let mesh_entity = app.world_mut().spawn(EffectShaderMeshOf(root)).id();

		app.update();

		assert_eq!(
			(
				Some(&MeshMaterial3d(handle)),
				Some(&Protected::<MeshMaterial3d<_Shader1>>::default())
			),
			(
				app.world()
					.entity(mesh_entity)
					.get::<MeshMaterial3d<_Shader1>>(),
				app.world()
					.entity(mesh_entity)
					.get::<Protected<MeshMaterial3d<_Shader1>>>()
			)
		)
	}

	#[test]
	fn pair_each_mesh_with_one_shader() {
		let mut app = setup();
		let shader1 = new_handle::<_Shader1>();
		let shader2 = new_handle::<_Shader2>();
		let shaders = EffectShaders {
			shaders: HashSet::from([
				EffectShaderHandle::from(shader1.clone()),
				EffectShaderHandle::from(shader2.clone()),
			]),
		};
		let root = app.world_mut().spawn(shaders).id();
		let mesh_entities = [
			app.world_mut().spawn(EffectShaderMeshOf(root)).id(),
			app.world_mut().spawn(EffectShaderMeshOf(root)).id(),
		];

		app.update();

		assert_eq!(
			(
				(
					Some(&MeshMaterial3d(shader1.clone())),
					Some(&MeshMaterial3d(shader2.clone()))
				),
				(
					Some(&MeshMaterial3d(shader1)),
					Some(&MeshMaterial3d(shader2))
				),
			),
			(
				(
					app.world()
						.entity(mesh_entities[0])
						.get::<MeshMaterial3d<_Shader1>>(),
					app.world()
						.entity(mesh_entities[0])
						.get::<MeshMaterial3d<_Shader2>>(),
				),
				(
					app.world()
						.entity(mesh_entities[1])
						.get::<MeshMaterial3d<_Shader1>>(),
					app.world()
						.entity(mesh_entities[1])
						.get::<MeshMaterial3d<_Shader2>>(),
				)
			)
		)
	}

	#[test]
	fn do_not_add_shaders_twice() {
		let mut app = setup();
		let shaders = EffectShaders {
			shaders: HashSet::from([EffectShaderHandle::from(new_handle::<_Shader1>())]),
		};
		let root = app.world_mut().spawn(shaders).id();
		let mesh_entity = app.world_mut().spawn(EffectShaderMeshOf(root)).id();

		app.update();

		app.world_mut()
			.entity_mut(mesh_entity)
			.remove::<MeshMaterial3d<_Shader1>>();

		app.update();

		assert_eq!(
			None,
			app.world()
				.entity(mesh_entity)
				.get::<MeshMaterial3d<_Shader1>>(),
		)
	}

	#[test]
	fn add_shaders_when_effect_shaders_changed() {
		let mut app = setup();
		let shader1 = new_handle::<_Shader1>();
		let shader2 = new_handle::<_Shader2>();
		let shaders = EffectShaders {
			shaders: HashSet::from([EffectShaderHandle::from(shader1)]),
		};
		let root = app.world_mut().spawn(shaders).id();
		let mesh_entity = app.world_mut().spawn(EffectShaderMeshOf(root)).id();

		app.update();

		app.world_mut()
			.entity_mut(root)
			.get_mut::<EffectShaders>()
			.unwrap()
			.shaders = HashSet::from([EffectShaderHandle::from(shader2.clone())]);

		app.update();

		assert_eq!(
			(None, Some(&MeshMaterial3d(shader2))),
			(
				app.world()
					.entity(mesh_entity)
					.get::<MeshMaterial3d<_Shader1>>(),
				app.world()
					.entity(mesh_entity)
					.get::<MeshMaterial3d<_Shader2>>(),
			)
		);
	}

	#[test]
	fn add_shaders_when_effect_shader_meshes_mutably_dereferenced() {
		let mut app = setup();
		let shader = new_handle::<_Shader1>();
		let shaders = EffectShaders {
			shaders: HashSet::from([EffectShaderHandle::from(shader.clone())]),
		};
		let root = app.world_mut().spawn(shaders).id();
		let mesh_entity = app.world_mut().spawn(EffectShaderMeshOf(root)).id();

		app.update();
		app.world_mut()
			.entity_mut(root)
			.get_mut::<EffectShaderMeshes>()
			.as_deref_mut();
		app.world_mut()
			.entity_mut(mesh_entity)
			.remove::<MeshMaterial3d<_Shader1>>();
		app.update();

		assert_eq!(
			Some(&MeshMaterial3d(shader)),
			app.world()
				.entity(mesh_entity)
				.get::<MeshMaterial3d<_Shader1>>(),
		);
	}
}
