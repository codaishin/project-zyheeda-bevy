use crate::{
	components::effect_shaders_target::{EffectShaderHandle, EffectShadersTarget},
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

pub(crate) fn instantiate_effect_shaders(
	mut commands: ZyheedaCommands,
	effect_shaders: Query<
		(Entity, &EffectShadersTarget, Option<&Active>),
		Changed<EffectShadersTarget>,
	>,
) {
	for (entity, effect_shaders, active) in &effect_shaders {
		clear(&mut commands, effect_shaders, active);
		instantiate(&mut commands, effect_shaders);
		commands.try_apply_on(&entity, |mut e| {
			e.try_insert(Active(effect_shaders.shaders.clone()));
		});
	}
}

#[derive(Component)]
pub(crate) struct Active(HashSet<EffectShaderHandle>);

fn clear(
	commands: &mut ZyheedaCommands,
	effect_shaders: &EffectShadersTarget,
	active: Option<&Active>,
) {
	let Some(Active(shaders)) = active else {
		return;
	};

	for shader in shaders {
		for entity in &effect_shaders.meshes {
			let Some(mut entity) = commands.get_mut(entity) else {
				continue;
			};

			entity.remove_protected_effect_shader(shader);
		}
	}
}

fn instantiate(commands: &mut ZyheedaCommands, effect_shaders: &EffectShadersTarget) {
	for shader in &effect_shaders.shaders {
		for entity in &effect_shaders.meshes {
			let Some(mut entity) = commands.get_mut(entity) else {
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
	use crate::components::effect_shaders_target::{EffectShaderHandle, EffectShadersTarget};
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
		let mesh_entity = app.world_mut().spawn_empty().id();
		let handle = new_handle::<_Shader1>();
		let shader = EffectShaderHandle::from(handle.clone());
		let shaders = EffectShadersTarget {
			meshes: HashSet::from([mesh_entity]),
			shaders: HashSet::from([shader]),
		};
		app.world_mut().spawn(shaders);

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
		let mesh_entities = [
			app.world_mut().spawn_empty().id(),
			app.world_mut().spawn_empty().id(),
		];
		let shader1 = new_handle::<_Shader1>();
		let shader2 = new_handle::<_Shader2>();
		let shaders = EffectShadersTarget {
			meshes: HashSet::from(mesh_entities),
			shaders: HashSet::from([
				EffectShaderHandle::from(shader1.clone()),
				EffectShaderHandle::from(shader2.clone()),
			]),
		};
		app.world_mut().spawn(shaders);

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
		let mesh_entity = app.world_mut().spawn_empty().id();
		let shaders = EffectShadersTarget {
			meshes: HashSet::from([mesh_entity]),
			shaders: HashSet::from([EffectShaderHandle::from(new_handle::<_Shader1>())]),
		};
		app.world_mut().spawn(shaders);

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
	fn add_shaders_when_effect_shaders_mutably_dereferenced() {
		let mut app = setup();
		let mesh_entity = app.world_mut().spawn_empty().id();
		let shader1 = new_handle::<_Shader1>();
		let shader2 = new_handle::<_Shader2>();
		let shaders = EffectShadersTarget {
			meshes: HashSet::from([mesh_entity]),
			shaders: HashSet::from([EffectShaderHandle::from(shader1)]),
		};
		let entity = app.world_mut().spawn(shaders).id();

		app.update();

		app.world_mut()
			.entity_mut(entity)
			.get_mut::<EffectShadersTarget>()
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
}
