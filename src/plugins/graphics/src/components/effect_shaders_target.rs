use crate::traits::{
	insert_protected_effect_shader::InsertProtectedEffectShader,
	remove_protected_effect_shader::RemoveProtectedEffectShader,
};
use bevy::{ecs::entity::EntityHashSet, prelude::*};
use common::{components::protected::Protected, zyheeda_commands::ZyheedaEntityCommands};
use std::{collections::HashSet, hash::Hash};

#[cfg(test)]
use bevy::asset::UntypedAssetId;

#[derive(Component, Default)]
pub(crate) struct EffectShaders {
	pub(crate) shaders: HashSet<EffectShaderHandle>,
}

#[derive(Component, Default)]
#[relationship_target(relationship = EffectShaderMeshOf)]
pub(crate) struct EffectShaderMeshes(EntityHashSet);

#[derive(Component)]
#[relationship(relationship_target = EffectShaderMeshes)]
pub(crate) struct EffectShaderMeshOf(pub(crate) Entity);

#[derive(Debug, Eq, Clone)]
pub(crate) struct EffectShaderHandle {
	handle: UntypedHandle,
	insert_into: fn(&mut ZyheedaEntityCommands, &UntypedHandle),
	remove_from: fn(&mut ZyheedaEntityCommands),
}

impl EffectShaderHandle {
	#[cfg(test)]
	pub(crate) fn id(&self) -> UntypedAssetId {
		self.handle.id()
	}

	fn insert_protected_handle<TMaterial: Asset + Material>(
		entity: &mut ZyheedaEntityCommands,
		handle: &UntypedHandle,
	) {
		entity.try_insert((
			MeshMaterial3d(handle.clone().typed::<TMaterial>()),
			Protected::<MeshMaterial3d<TMaterial>>::default(),
		));
	}

	fn remove_protected_handle<TMaterial: Asset + Material>(entity: &mut ZyheedaEntityCommands) {
		entity.try_remove::<(
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

impl InsertProtectedEffectShader for ZyheedaEntityCommands<'_> {
	fn insert_protected_effect_shader(&mut self, effect_shader: &EffectShaderHandle) {
		let insert_into = effect_shader.insert_into;
		let handle = &effect_shader.handle;
		insert_into(self, handle)
	}
}

impl RemoveProtectedEffectShader for ZyheedaEntityCommands<'_> {
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
	use common::{traits::accessors::get::GetMut, zyheeda_commands::ZyheedaCommands};
	use testing::new_handle;

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
		mut commands: ZyheedaCommands,
		entities: Query<Entity>,
	) {
		for entity in &entities {
			let Some(mut entity) = commands.get_mut(&entity) else {
				continue;
			};

			entity.insert_protected_effect_shader(&shader)
		}
	}

	fn remove_shader_system(
		In(shader): In<EffectShaderHandle>,
		mut commands: ZyheedaCommands,
		entities: Query<Entity>,
	) {
		for entity in &entities {
			let Some(mut entity) = commands.get_mut(&entity) else {
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
