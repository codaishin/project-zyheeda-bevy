use crate::traits::{
	insert_protected_effect_shader::InsertProtectedEffectShader,
	remove_protected_effect_shader::RemoveProtectedEffectShader,
};
use bevy::{ecs::system::EntityCommands, prelude::*};
use common::{
	components::Protected,
	traits::track::{IsTracking, Track, Untrack},
};
use std::collections::HashSet;

#[cfg(test)]
use bevy::asset::UntypedAssetId;

#[derive(Component, Default)]
pub struct EffectShadersTarget {
	pub(crate) meshes: HashSet<Entity>,
	pub(crate) shaders: HashSet<EffectShaderHandle>,
}

impl Track<Handle<Mesh>> for EffectShadersTarget {
	fn track(&mut self, entity: Entity, _: &Handle<Mesh>) {
		self.meshes.insert(entity);
	}
}

impl IsTracking<Handle<Mesh>> for EffectShadersTarget {
	fn is_tracking(&self, entity: &Entity) -> bool {
		self.meshes.contains(entity)
	}
}

impl Untrack<Handle<Mesh>> for EffectShadersTarget {
	fn untrack(&mut self, entity: &Entity) {
		self.meshes.remove(entity);
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) struct EffectShaderHandle {
	handle: UntypedHandle,
	insert_into: fn(&mut EntityCommands, &UntypedHandle),
	remove_from: fn(&mut EntityCommands),
}

impl EffectShaderHandle {
	#[cfg(test)]
	pub(crate) fn id(&self) -> UntypedAssetId {
		self.handle.id()
	}

	fn insert_protected_handle<TMaterial: Asset + Material>(
		entity: &mut EntityCommands,
		handle: &UntypedHandle,
	) {
		entity.insert((
			handle.clone().typed::<TMaterial>(),
			Protected::<Handle<TMaterial>>::default(),
		));
	}

	fn remove_protected_handle<TMaterial: Asset + Material>(entity: &mut EntityCommands) {
		entity.remove::<(Handle<TMaterial>, Protected<Handle<TMaterial>>)>();
	}
}

impl InsertProtectedEffectShader for EntityCommands<'_> {
	fn insert_protected_effect_shader(&mut self, effect_shader: &EffectShaderHandle) {
		let insert_into = effect_shader.insert_into;
		let handle = &effect_shader.handle;
		insert_into(self, handle)
	}
}

impl RemoveProtectedEffectShader for EntityCommands<'_> {
	fn remove_protected_effect_shader(&mut self, effect_shader: &EffectShaderHandle) {
		let remove_from = effect_shader.remove_from;
		remove_from(self)
	}
}

impl<TMaterial: Asset + Material> From<Handle<TMaterial>> for EffectShaderHandle {
	fn from(handle: Handle<TMaterial>) -> Self {
		Self {
			handle: handle.untyped(),
			insert_into: EffectShaderHandle::insert_protected_handle::<TMaterial>,
			remove_from: EffectShaderHandle::remove_protected_handle::<TMaterial>,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{ecs::system::RunSystemOnce, render::render_resource::AsBindGroup};
	use common::{components::Protected, test_tools::utils::new_handle};

	#[test]
	fn push_mesh_handle() {
		let mut shader = EffectShadersTarget::default();
		let entity = Entity::from_raw(42);

		shader.track(entity, &new_handle());

		assert_eq!(HashSet::from([entity]), shader.meshes);
	}

	#[test]
	fn push_mesh_handles() {
		let mut shader = EffectShadersTarget::default();
		let entities = [Entity::from_raw(11), Entity::from_raw(66)];

		for entity in &entities {
			shader.track(*entity, &new_handle());
		}

		assert_eq!(HashSet::from(entities), shader.meshes);
	}

	#[test]
	fn remove_mesh_handles() {
		let mut shader = EffectShadersTarget {
			meshes: HashSet::from([Entity::from_raw(11), Entity::from_raw(66)]),
			..default()
		};

		shader.untrack(&Entity::from_raw(66));

		assert_eq!(HashSet::from([Entity::from_raw(11)]), shader.meshes);
	}

	#[test]
	fn contains_mesh_handles() {
		let shader = EffectShadersTarget {
			meshes: HashSet::from([Entity::from_raw(11)]),
			..default()
		};

		assert_eq!(
			[true, false],
			[
				shader.is_tracking(&Entity::from_raw(11)),
				shader.is_tracking(&Entity::from_raw(12))
			]
		);
	}

	#[derive(Asset, TypePath, Clone, AsBindGroup)]
	struct _Asset {}

	impl Material for _Asset {}

	#[test]
	fn effect_shader_id() {
		let handle = new_handle::<_Asset>();

		let shader = EffectShaderHandle::from(handle.clone());

		assert_eq!(handle.id().untyped(), shader.id());
	}

	fn insert_shader_system(
		In(shader): In<EffectShaderHandle>,
		mut commands: Commands,
		entities: Query<Entity>,
	) {
		for entity in &entities {
			let Some(mut entity) = commands.get_entity(entity) else {
				continue;
			};

			entity.insert_protected_effect_shader(&shader)
		}
	}

	fn remove_shader_system(
		In(shader): In<EffectShaderHandle>,
		mut commands: Commands,
		entities: Query<Entity>,
	) {
		for entity in &entities {
			let Some(mut entity) = commands.get_entity(entity) else {
				continue;
			};

			entity.remove_protected_effect_shader(&shader)
		}
	}

	#[test]
	fn insert_effect_shader() {
		let mut app = App::new();
		let handle = new_handle::<_Asset>();
		let shader = EffectShaderHandle::from(handle.clone());
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once_with(shader, insert_shader_system);

		assert_eq!(
			Some(&handle),
			app.world().entity(entity).get::<Handle<_Asset>>()
		);
	}

	#[test]
	fn insert_protected_effect_shader() {
		let mut app = App::new();
		let shader = EffectShaderHandle::from(new_handle::<_Asset>());
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once_with(shader, insert_shader_system);

		assert_eq!(
			Some(&Protected::<Handle<_Asset>>::default()),
			app.world()
				.entity(entity)
				.get::<Protected<Handle<_Asset>>>()
		);
	}

	#[test]
	fn remove_inserted_components() {
		let mut app = App::new();
		let shader = EffectShaderHandle::from(new_handle::<_Asset>());
		let entity = app
			.world_mut()
			.spawn((
				new_handle::<_Asset>(),
				Protected::<Handle<_Asset>>::default(),
			))
			.id();

		app.world_mut()
			.run_system_once_with(shader, remove_shader_system);

		assert_eq!(
			(None, None),
			(
				app.world().entity(entity).get::<Handle<_Asset>>(),
				app.world()
					.entity(entity)
					.get::<Protected<Handle<_Asset>>>(),
			)
		);
	}
}
