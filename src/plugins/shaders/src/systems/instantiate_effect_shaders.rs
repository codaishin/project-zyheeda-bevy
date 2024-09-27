use crate::{
	components::effect_shader::{EffectShader, EffectShaders},
	traits::{
		insert_unmovable_effect_shader::InsertUnmovableEffectShader,
		remove_unmovable_effect_shader::RemoveUnmovableEffectShader,
	},
};
use bevy::prelude::*;
use common::traits::try_insert_on::TryInsertOn;

pub(crate) fn instantiate_effect_shaders(
	mut commands: Commands,
	effect_shaders: Query<(Entity, &EffectShaders, Option<&Active>), Changed<EffectShaders>>,
) {
	for (entity, effect_shaders, active) in &effect_shaders {
		clear(&mut commands, effect_shaders, active);
		instantiate(&mut commands, effect_shaders);
		commands.try_insert_on(entity, Active(effect_shaders.shaders.clone()));
	}
}

#[derive(Component)]
pub(crate) struct Active(Vec<EffectShader>);

fn clear(commands: &mut Commands, effect_shaders: &EffectShaders, active: Option<&Active>) {
	let Some(Active(shaders)) = active else {
		return;
	};

	for shader in shaders {
		for entity in &effect_shaders.meshes {
			let Some(mut entity) = commands.get_entity(*entity) else {
				continue;
			};

			entity.remove_unmovable_effect_shader(shader);
		}
	}
}

fn instantiate(commands: &mut Commands, effect_shaders: &EffectShaders) {
	for shader in &effect_shaders.shaders {
		for entity in &effect_shaders.meshes {
			let Some(mut entity) = commands.get_entity(*entity) else {
				continue;
			};

			entity.insert_unmovable_effect_shader(shader);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::effect_shader::{EffectShader, EffectShaders};
	use common::{
		components::Unmovable,
		test_tools::utils::{new_handle, SingleThreadedApp},
	};

	#[derive(Asset, TypePath)]
	struct _Shader1;

	#[derive(Asset, TypePath)]
	struct _Shader2;

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
			meshes: vec![mesh_entity],
			shaders: vec![shader],
		};
		app.world_mut().spawn(shaders);

		app.update();

		assert_eq!(
			(
				Some(&handle),
				Some(&Unmovable::<Handle<_Shader1>>::default())
			),
			(
				app.world().entity(mesh_entity).get::<Handle<_Shader1>>(),
				app.world()
					.entity(mesh_entity)
					.get::<Unmovable<Handle<_Shader1>>>()
			)
		)
	}

	#[test]
	fn pair_each_mesh_with_one_shader() {
		let mut app = setup();
		let mesh_entities = vec![
			app.world_mut().spawn_empty().id(),
			app.world_mut().spawn_empty().id(),
		];
		let shader1 = new_handle::<_Shader1>();
		let shader2 = new_handle::<_Shader2>();
		let shaders = EffectShaders {
			meshes: mesh_entities.clone(),
			shaders: vec![
				EffectShader::from(shader1.clone()),
				EffectShader::from(shader2.clone()),
			],
		};
		app.world_mut().spawn(shaders);

		app.update();

		assert_eq!(
			(
				(Some(&shader1), Some(&shader2)),
				(Some(&shader1), Some(&shader2)),
			),
			(
				(
					app.world()
						.entity(mesh_entities[0])
						.get::<Handle<_Shader1>>(),
					app.world()
						.entity(mesh_entities[0])
						.get::<Handle<_Shader2>>(),
				),
				(
					app.world()
						.entity(mesh_entities[1])
						.get::<Handle<_Shader1>>(),
					app.world()
						.entity(mesh_entities[1])
						.get::<Handle<_Shader2>>(),
				)
			)
		)
	}

	#[test]
	fn do_not_add_shaders_twice() {
		let mut app = setup();
		let mesh_entity = app.world_mut().spawn_empty().id();
		let shaders = EffectShaders {
			meshes: vec![mesh_entity],
			shaders: vec![EffectShader::from(new_handle::<_Shader1>())],
		};
		app.world_mut().spawn(shaders);

		app.update();

		app.world_mut()
			.entity_mut(mesh_entity)
			.remove::<Handle<_Shader1>>();

		app.update();

		assert_eq!(
			None,
			app.world().entity(mesh_entity).get::<Handle<_Shader1>>(),
		)
	}

	#[test]
	fn add_shaders_when_effect_shaders_mutably_dereferenced() {
		let mut app = setup();
		let mesh_entity = app.world_mut().spawn_empty().id();
		let shader1 = new_handle::<_Shader1>();
		let shader2 = new_handle::<_Shader2>();
		let shaders = EffectShaders {
			meshes: vec![mesh_entity],
			shaders: vec![EffectShader::from(shader1)],
		};
		let entity = app.world_mut().spawn(shaders).id();

		app.update();

		app.world_mut()
			.entity_mut(entity)
			.get_mut::<EffectShaders>()
			.unwrap()
			.shaders = vec![EffectShader::from(shader2.clone())];

		app.update();

		assert_eq!(
			(None, Some(&shader2)),
			(
				app.world().entity(mesh_entity).get::<Handle<_Shader1>>(),
				app.world().entity(mesh_entity).get::<Handle<_Shader2>>(),
			)
		);
	}
}
