use crate::{components::child_meshes::ChildMeshes, materials::lit_material::StandardLitMaterial};
use bevy::{ecs::component::Mutable, prelude::*};
use common::prelude::*;

type DataOrMeshesChanged<TData> = Or<(Changed<TData>, Changed<ChildMeshes>)>;

impl<T> PropagateMaterial for T where T: Component {}

pub trait PropagateMaterial: Component + Sized {
	fn propagate_material<TMaterial>(
		mut commands: ZyheedaCommands,
		meshes: Query<(&Self, &mut Visibility, &ChildMeshes), DataOrMeshesChanged<Self>>,
		effect_materials: Query<&MeshMaterial3d<TMaterial>>,
		mut assets: ResMut<Assets<TMaterial>>,
		mut buffers: ResMut<TMaterial::TBuffer>,
	) where
		TMaterial: Material + Default + UpdateMaterial<TData = Self>,
	{
		for (data, mut visibility, child_meshes) in meshes {
			for entity in child_meshes.iter() {
				commands.try_apply_on(&entity, |mut e| {
					e.try_remove::<MeshMaterial3d<StandardMaterial>>();
					e.try_remove::<MeshMaterial3d<StandardLitMaterial>>();

					{
						let effect_material = effect_materials
							.get(entity)
							.map(|MeshMaterial3d(id)| assets.get_mut(id));

						if let Ok(Some(mut effect_material)) = effect_material {
							effect_material.update_material(&mut buffers, data);
							return;
						};
					}

					e.try_insert(MeshMaterial3d(
						assets.add(TMaterial::from_data(&mut buffers, data)),
					));
				});
			}
			*visibility = Visibility::Visible;
		}
	}
}

pub(crate) trait UpdateMaterial {
	type TData: Component;
	type TBuffer: Resource<Mutability = Mutable>;

	fn update_material(&mut self, buffers: &mut Self::TBuffer, data: &Self::TData);
}

trait FromData: Default + UpdateMaterial {
	fn from_data(buffers: &mut Self::TBuffer, data: &Self::TData) -> Self {
		let mut value = Self::default();

		value.update_material(buffers, data);

		value
	}
}

impl<T> FromData for T where T: Default + UpdateMaterial {}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::components::child_meshes::ChildMeshOf;
	use bevy::render::render_resource::AsBindGroup;
	use testing::{SingleThreadedApp, new_handle};

	#[derive(Component, Debug, PartialEq, Clone)]
	#[require(Visibility)]
	struct _Data;

	#[derive(Asset, TypePath, AsBindGroup, Debug, PartialEq, Clone)]
	struct _Material {
		from_default: bool,
		data: Option<_Data>,
	}

	impl Default for _Material {
		fn default() -> Self {
			Self {
				from_default: true,
				data: None,
			}
		}
	}

	impl Material for _Material {}

	impl UpdateMaterial for _Material {
		type TData = _Data;
		type TBuffer = _Buffers;

		fn update_material(&mut self, _: &mut Self::TBuffer, data: &_Data) {
			self.data = Some(data.clone());
		}
	}

	#[derive(Resource, Default)]
	struct _Buffers;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<_Buffers>();
		app.init_resource::<Assets<_Material>>();
		app.add_systems(Update, _Data::propagate_material::<_Material>);

		app
	}

	#[test]
	fn add_material() {
		let mut app = setup();
		let entity = app.world_mut().spawn(_Data).id();
		let child = app.world_mut().spawn(ChildMeshOf(entity)).id();

		app.update();

		assert!(
			app.world()
				.entity(child)
				.contains::<MeshMaterial3d<_Material>>(),
		);
	}

	#[test]
	fn set_material_data() {
		let mut app = setup();
		let entity = app.world_mut().spawn(_Data).id();
		let child = app.world_mut().spawn(ChildMeshOf(entity)).id();

		app.update();

		let MeshMaterial3d(handle) = app
			.world()
			.entity(child)
			.get::<MeshMaterial3d<_Material>>()
			.unwrap();
		assert_eq!(
			Some(&_Material {
				data: Some(_Data),
				..default()
			}),
			app.world().resource::<Assets<_Material>>().get(handle),
		);
	}

	#[test]
	fn reuse_material_handle() {
		let old_handle = new_handle();
		let mut app = setup();
		let entity = app.world_mut().spawn(_Data).id();
		let child = app
			.world_mut()
			.spawn((
				ChildMeshOf(entity),
				MeshMaterial3d::<_Material>(old_handle.clone()),
			))
			.id();
		_ = app.world_mut().resource_mut::<Assets<_Material>>().insert(
			&old_handle,
			_Material {
				from_default: false,
				..default()
			},
		);

		app.update();

		let MeshMaterial3d(handle) = app
			.world()
			.entity(child)
			.get::<MeshMaterial3d<_Material>>()
			.unwrap();
		assert_eq!(
			(
				Some(&_Material {
					data: Some(_Data),
					from_default: false,
				}),
				&old_handle
			),
			(
				app.world().resource::<Assets<_Material>>().get(&old_handle),
				handle
			)
		);
	}

	#[test]
	fn set_material_data_if_old_handle_invalid() {
		let old_handle = new_handle();
		let mut app = setup();
		let entity = app.world_mut().spawn(_Data).id();
		let child = app
			.world_mut()
			.spawn((
				ChildMeshOf(entity),
				MeshMaterial3d::<_Material>(old_handle.clone()),
			))
			.id();

		app.update();

		let MeshMaterial3d(handle) = app
			.world()
			.entity(child)
			.get::<MeshMaterial3d<_Material>>()
			.unwrap();
		assert_eq!(
			Some(&_Material {
				data: Some(_Data),
				..default()
			}),
			app.world().resource::<Assets<_Material>>().get(handle),
		);
	}

	#[test]
	fn drop_old_handle_if_invalid() {
		let old_handle = new_handle();
		let mut app = setup();
		let entity = app.world_mut().spawn(_Data).id();
		let child = app
			.world_mut()
			.spawn((
				ChildMeshOf(entity),
				MeshMaterial3d::<_Material>(old_handle.clone()),
			))
			.id();

		app.update();

		let MeshMaterial3d(handle) = app
			.world()
			.entity(child)
			.get::<MeshMaterial3d<_Material>>()
			.unwrap();
		assert_ne!(handle, &old_handle);
	}

	#[test]
	fn remove_standard_material() {
		let mut app = setup();
		let entity = app.world_mut().spawn(_Data).id();
		let child = app
			.world_mut()
			.spawn((
				ChildMeshOf(entity),
				MeshMaterial3d(new_handle::<StandardMaterial>()),
			))
			.id();

		app.update();

		assert_eq!(
			None,
			app.world()
				.entity(child)
				.get::<MeshMaterial3d<StandardMaterial>>(),
		);
	}

	#[test]
	fn remove_standard_lit_material() {
		let mut app = setup();
		let entity = app.world_mut().spawn(_Data).id();
		let child = app
			.world_mut()
			.spawn((
				ChildMeshOf(entity),
				MeshMaterial3d(new_handle::<StandardLitMaterial>()),
			))
			.id();

		app.update();

		assert_eq!(
			None,
			app.world()
				.entity(child)
				.get::<MeshMaterial3d<StandardLitMaterial>>(),
		);
	}

	#[test]
	fn set_visibility_to_visible() {
		let mut app = setup();
		let entity = app.world_mut().spawn(_Data).id();
		app.world_mut().spawn((
			ChildMeshOf(entity),
			MeshMaterial3d(new_handle::<StandardMaterial>()),
		));

		app.update();

		assert_eq!(
			Some(&Visibility::Visible),
			app.world().entity(entity).get::<Visibility>(),
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		let entity = app.world_mut().spawn(_Data).id();
		let child = app.world_mut().spawn(ChildMeshOf(entity)).id();

		app.update();
		app.world_mut()
			.entity_mut(child)
			.remove::<MeshMaterial3d<_Material>>();
		app.update();

		assert_eq!(
			None,
			app.world().entity(child).get::<MeshMaterial3d<_Material>>(),
		);
	}

	#[test]
	fn act_again_if_children_changed() {
		let mut app = setup();
		let entity = app.world_mut().spawn(_Data).id();
		let child = app.world_mut().spawn(ChildMeshOf(entity)).id();

		app.update();
		app.world_mut()
			.entity_mut(child)
			.remove::<MeshMaterial3d<_Material>>();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<ChildMeshes>()
			.as_deref_mut();
		app.update();

		assert!(
			app.world()
				.entity(child)
				.contains::<MeshMaterial3d<_Material>>(),
		);
	}

	#[test]
	fn act_again_if_effect_data_changed() {
		let mut app = setup();
		let entity = app.world_mut().spawn(_Data).id();
		let child = app.world_mut().spawn(ChildMeshOf(entity)).id();

		app.update();
		app.world_mut()
			.entity_mut(child)
			.remove::<MeshMaterial3d<_Material>>();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<_Data>()
			.as_deref_mut();
		app.update();

		assert!(
			app.world()
				.entity(child)
				.contains::<MeshMaterial3d<_Material>>(),
		);
	}
}
