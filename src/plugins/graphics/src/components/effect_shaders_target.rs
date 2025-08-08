use crate::traits::{
	insert_protected_effect_shader::InsertProtectedEffectShader,
	remove_protected_effect_shader::RemoveProtectedEffectShader,
};
use bevy::{ecs::system::EntityCommands, prelude::*};
use common::{
	components::protected::Protected,
	traits::track::{IsTracking, Track, Untrack},
};
use std::{collections::HashSet, hash::Hash};

#[cfg(test)]
use bevy::asset::UntypedAssetId;

#[derive(Component, Default)]
pub struct EffectShadersTarget {
	pub(crate) meshes: HashSet<Entity>,
	pub(crate) shaders: HashSet<EffectShaderHandle>,
}

impl Track<Mesh3d> for EffectShadersTarget {
	fn track(&mut self, entity: Entity, _: &Mesh3d) {
		self.meshes.insert(entity);
	}
}

impl IsTracking<Mesh3d> for EffectShadersTarget {
	fn is_tracking(&self, entity: &Entity) -> bool {
		self.meshes.contains(entity)
	}
}

impl Untrack<Mesh3d> for EffectShadersTarget {
	fn untrack(&mut self, entity: &Entity) {
		self.meshes.remove(entity);
	}
}

#[derive(Debug, Eq, Clone)]
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
			MeshMaterial3d(handle.clone().typed::<TMaterial>()),
			Protected::<MeshMaterial3d<TMaterial>>::default(),
		));
	}

	fn remove_protected_handle<TMaterial: Asset + Material>(entity: &mut EntityCommands) {
		entity.remove::<(
			MeshMaterial3d<TMaterial>,
			Protected<MeshMaterial3d<TMaterial>>,
		)>();
	}
}

impl PartialEq for EffectShaderHandle {
	fn eq(&self, other: &Self) -> bool {
		self.handle == other.handle
			&& std::ptr::fn_addr_eq(self.insert_into, other.insert_into)
			&& std::ptr::fn_addr_eq(self.remove_from, other.remove_from)
	}
}

impl Hash for EffectShaderHandle {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.handle.hash(state);
		self.insert_into.hash(state);
		self.remove_from.hash(state);
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
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		render::render_resource::AsBindGroup,
	};
	use testing::new_handle;

	#[test]
	fn push_mesh_handle() {
		let mut shader = EffectShadersTarget::default();
		let entity = Entity::from_raw(42);

		shader.track(entity, &Mesh3d(new_handle()));

		assert_eq!(HashSet::from([entity]), shader.meshes);
	}

	#[test]
	fn push_mesh_handles() {
		let mut shader = EffectShadersTarget::default();
		let entities = [Entity::from_raw(11), Entity::from_raw(66)];

		for entity in &entities {
			shader.track(*entity, &Mesh3d(new_handle()));
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

	#[derive(Asset, TypePath, Clone, AsBindGroup, Debug, PartialEq)]
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
			let Ok(mut entity) = commands.get_entity(entity) else {
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
			let Ok(mut entity) = commands.get_entity(entity) else {
				continue;
			};

			entity.remove_protected_effect_shader(&shader)
		}
	}

	#[test]
	fn insert_effect_shader() -> Result<(), RunSystemError> {
		let mut app = App::new();
		let handle = new_handle::<_Asset>();
		let shader = EffectShaderHandle::from(handle.clone());
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once_with(insert_shader_system, shader)?;

		assert_eq!(
			Some(&MeshMaterial3d(handle)),
			app.world().entity(entity).get::<MeshMaterial3d<_Asset>>()
		);
		Ok(())
	}

	#[test]
	fn insert_protected_effect_shader() -> Result<(), RunSystemError> {
		let mut app = App::new();
		let shader = EffectShaderHandle::from(new_handle::<_Asset>());
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once_with(insert_shader_system, shader)?;

		assert_eq!(
			Some(&Protected::<MeshMaterial3d<_Asset>>::default()),
			app.world()
				.entity(entity)
				.get::<Protected<MeshMaterial3d<_Asset>>>()
		);
		Ok(())
	}

	#[test]
	fn remove_inserted_components() -> Result<(), RunSystemError> {
		let mut app = App::new();
		let shader = EffectShaderHandle::from(new_handle::<_Asset>());
		let entity = app
			.world_mut()
			.spawn((
				MeshMaterial3d(new_handle::<_Asset>()),
				Protected::<MeshMaterial3d<_Asset>>::default(),
			))
			.id();

		app.world_mut()
			.run_system_once_with(remove_shader_system, shader)?;

		assert_eq!(
			(None, None),
			(
				app.world().entity(entity).get::<MeshMaterial3d<_Asset>>(),
				app.world()
					.entity(entity)
					.get::<Protected<MeshMaterial3d<_Asset>>>(),
			)
		);
		Ok(())
	}
}
