use crate::{
	components::effect_shader::EffectShaders,
	traits::insert_effect_shader::InsertEffectShader,
};
use bevy::prelude::*;
use common::traits::try_insert_on::TryInsertOn;

type Components<'a> = (
	Entity,
	&'a EffectShaders,
	Option<&'a EffectShadersController>,
	Option<&'a Children>,
);

#[derive(Component, Default, Clone)]
pub struct EffectShadersController(Vec<Entity>);

impl EffectShadersController {
	pub(crate) fn instantiate_shaders(
		mut commands: Commands,
		effect_shaders: Query<Components, Changed<EffectShaders>>,
	) {
		for (entity, shaders, controller, children) in &effect_shaders {
			let mut controller = controller.cloned().unwrap_or_default();

			controller.clear(&mut commands, children);
			controller.instantiate(&mut commands, shaders, entity);

			commands.try_insert_on(entity, controller);
		}
	}

	fn clear(&self, commands: &mut Commands, children: Option<&Children>) {
		let Some(children) = children else {
			return;
		};

		let EffectShadersController(added) = self;

		for child in children.iter().filter(|child| added.contains(child)) {
			let Some(child) = commands.get_entity(*child) else {
				continue;
			};

			child.despawn_recursive();
		}
	}

	fn instantiate(&mut self, commands: &mut Commands, shaders: &EffectShaders, entity: Entity) {
		let EffectShadersController(added) = self;

		for mesh in &shaders.meshes {
			let mut child = commands.spawn_empty();
			added.push(child.id());
			child.set_parent(entity);

			child.insert(mesh.clone());
			for shader in &shaders.shaders {
				child.insert_effect_shader(shader);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::effect_shader::{EffectShader, EffectShaders};
	use common::test_tools::utils::{new_handle, SingleThreadedApp};
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

	fn find_children(app: &mut App, entity: Entity) -> impl Iterator<Item = EntityRef> {
		app.world().iter_entities().filter(child_of(entity))
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, EffectShadersController::instantiate_shaders);

		app
	}

	#[test]
	fn insert_single_shader_effect() {
		let mut app = setup();
		let handle = new_handle::<_Shader1>();
		let shader = EffectShader::from(handle.clone());
		let shaders = EffectShaders {
			meshes: vec![new_handle()],
			shaders: vec![shader],
		};
		let entity = app.world_mut().spawn(shaders).id();

		app.update();

		assert_eq!(
			vec![Some(&handle)],
			find_children(&mut app, entity)
				.map(|child| child.get::<Handle<_Shader1>>())
				.collect::<Vec<_>>()
		)
	}

	#[test]
	fn insert_single_mesh_copy() {
		let mut app = setup();
		let handle = new_handle::<Mesh>();
		let shader = EffectShader::from(new_handle::<_Shader1>());
		let shaders = EffectShaders {
			meshes: vec![handle.clone()],
			shaders: vec![shader],
		};
		let entity = app.world_mut().spawn(shaders).id();

		app.update();

		assert_eq!(
			vec![Some(&handle)],
			find_children(&mut app, entity)
				.map(|child| child.get::<Handle<Mesh>>())
				.collect::<Vec<_>>()
		)
	}

	#[test]
	fn pair_effect_shaders_with_each_mesh() {
		let mut app = setup();
		let meshes = vec![new_handle(), new_handle()];
		let shader1 = new_handle::<_Shader1>();
		let shader2 = new_handle::<_Shader2>();
		let shaders = EffectShaders {
			meshes: meshes.clone(),
			shaders: vec![
				EffectShader::from(shader1.clone()),
				EffectShader::from(shader2.clone()),
			],
		};
		let entity = app.world_mut().spawn(shaders).id();

		app.update();

		assert_eq!(
			HashSet::from([
				(Some(&meshes[0]), Some(&shader1), Some(&shader2)),
				(Some(&meshes[1]), Some(&shader1), Some(&shader2)),
			]),
			find_children(&mut app, entity)
				.map(|child| (
					child.get::<Handle<Mesh>>(),
					child.get::<Handle<_Shader1>>(),
					child.get::<Handle<_Shader2>>(),
				))
				.collect::<HashSet<_>>()
		)
	}

	#[test]
	fn do_not_spawn_children_twice() {
		let mut app = setup();
		let shaders = EffectShaders {
			meshes: vec![new_handle()],
			shaders: vec![EffectShader::from(new_handle::<_Shader1>())],
		};
		let entity = app.world_mut().spawn(shaders).id();

		app.update();
		app.update();

		assert_eq!(
			vec![(true, true)],
			find_children(&mut app, entity)
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
		let shaders = EffectShaders {
			meshes: vec![new_handle()],
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
			find_children(&mut app, entity)
				.map(|child| (
					child.contains::<Handle<Mesh>>(),
					child.contains::<Handle<_Shader1>>(),
					child.contains::<Handle<_Shader2>>(),
				))
				.collect::<Vec<_>>()
		)
	}

	#[test]
	fn when_rewriting_despawn_children_recursively() {
		#[derive(Component, Debug, PartialEq)]
		struct _DeepChild;

		let mut app = setup();
		let shaders = EffectShaders {
			meshes: vec![new_handle()],
			shaders: vec![EffectShader::from(new_handle::<_Shader1>())],
		};
		let entity = app.world_mut().spawn(shaders).id();

		app.update();

		app.world_mut()
			.entity_mut(entity)
			.get_mut::<EffectShaders>()
			.unwrap()
			.shaders = vec![EffectShader::from(new_handle::<_Shader2>())];
		let child = find_children(&mut app, entity)
			.map(|e| e.id())
			.next()
			.unwrap();
		app.world_mut().spawn(_DeepChild).set_parent(child);

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
		let shaders = EffectShaders {
			meshes: vec![new_handle()],
			shaders: vec![EffectShader::from(new_handle::<_Shader1>())],
		};
		let entity = app.world_mut().spawn(shaders).id();

		app.update();

		app.world_mut()
			.entity_mut(entity)
			.get_mut::<EffectShaders>()
			.unwrap()
			.shaders = vec![EffectShader::from(new_handle::<_Shader2>())];
		let child = app.world_mut().spawn_empty().set_parent(entity).id();

		app.update();

		assert!(app.world().get_entity(child).is_some());
	}
}
