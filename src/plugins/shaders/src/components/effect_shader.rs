use crate::traits::insert_unmovable_effect_shader::InsertUnmovableEffectShader;
use bevy::{ecs::system::EntityCommands, prelude::*};
use common::{components::Unmovable, traits::push_component::PushComponent};

#[cfg(test)]
use bevy::asset::UntypedAssetId;

#[derive(Component, Default)]
pub struct EffectShaders {
	pub(crate) meshes: Vec<MeshData>,
	pub(crate) shaders: Vec<EffectShader>,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct MeshData {
	pub(crate) entity: Entity,
	pub(crate) handle: Handle<Mesh>,
}

impl PushComponent<Handle<Mesh>> for EffectShaders {
	fn push_component(&mut self, entity: Entity, handle: Handle<Mesh>) {
		self.meshes.push(MeshData { entity, handle });
	}
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct EffectShader {
	handle: UntypedHandle,
	insert_into: fn(&mut EntityCommands, &UntypedHandle),
}

impl EffectShader {
	#[cfg(test)]
	pub(crate) fn id(&self) -> UntypedAssetId {
		self.handle.id()
	}

	fn insert_as_unmovable_handle<TAsset: Asset>(
		entity: &mut EntityCommands,
		handle: &UntypedHandle,
	) {
		entity.insert((
			handle.clone().typed::<TAsset>(),
			Unmovable::<Handle<TAsset>>::default(),
		));
	}
}

impl<'a> InsertUnmovableEffectShader for EntityCommands<'a> {
	fn insert_unmovable_effect_shader(&mut self, effect_shader: &EffectShader) {
		let insert_into = effect_shader.insert_into;
		let handle = &effect_shader.handle;
		insert_into(self, handle);
	}
}

impl<TAsset: Asset> From<Handle<TAsset>> for EffectShader {
	fn from(handle: Handle<TAsset>) -> Self {
		Self {
			handle: handle.untyped(),
			insert_into: EffectShader::insert_as_unmovable_handle::<TAsset>,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::RunSystemOnce;
	use common::{components::Unmovable, test_tools::utils::new_handle};

	#[test]
	fn push_mesh_handle() {
		let mut shader = EffectShaders::default();
		let entity = Entity::from_raw(42);
		let handle = new_handle::<Mesh>();

		shader.push_component(entity, handle.clone());

		assert_eq!(vec![MeshData { entity, handle }], shader.meshes);
	}

	#[test]
	fn push_mesh_handles() {
		let mut shader = EffectShaders::default();
		let meshes = vec![
			MeshData {
				entity: Entity::from_raw(11),
				handle: new_handle::<Mesh>(),
			},
			MeshData {
				entity: Entity::from_raw(66),
				handle: new_handle::<Mesh>(),
			},
		];

		for MeshData { entity, handle } in &meshes {
			shader.push_component(*entity, handle.clone());
		}

		assert_eq!(meshes, shader.meshes);
	}

	#[derive(Asset, TypePath)]
	struct _Asset;

	#[test]
	fn effect_shader_id() {
		let handle = new_handle::<_Asset>();

		let shader = EffectShader::from(handle.clone());

		assert_eq!(handle.id().untyped(), shader.id());
	}

	fn insert_shader_system(
		In(shader): In<EffectShader>,
		mut commands: Commands,
		entities: Query<Entity>,
	) {
		for entity in &entities {
			let Some(mut entity) = commands.get_entity(entity) else {
				continue;
			};
			entity.insert_unmovable_effect_shader(&shader);
		}
	}

	#[test]
	fn insert_effect_shader() {
		let mut app = App::new();
		let handle = new_handle::<_Asset>();
		let shader = EffectShader::from(handle.clone());
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once_with(shader, insert_shader_system);

		assert_eq!(
			Some(&handle),
			app.world().entity(entity).get::<Handle<_Asset>>()
		);
	}

	#[test]
	fn insert_effect_shader_unmovable() {
		let mut app = App::new();
		let shader = EffectShader::from(new_handle::<_Asset>());
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once_with(shader, insert_shader_system);

		assert_eq!(
			Some(&Unmovable::<Handle<_Asset>>::default()),
			app.world()
				.entity(entity)
				.get::<Unmovable<Handle<_Asset>>>()
		);
	}
}
