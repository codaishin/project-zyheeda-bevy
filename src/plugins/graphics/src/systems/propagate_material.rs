use crate::{components::child_meshes::ChildMeshes, materials::lit_material::StandardLitMaterial};
use bevy::prelude::*;
use common::prelude::*;

type DataOrMeshesChanged<TData> = Or<(Changed<TData>, Changed<ChildMeshes>)>;

impl<T> PropagateMaterial for T where T: Component {}

pub trait PropagateMaterial: Component + Sized {
	fn propagate_material<TMaterial>(
		mut commands: ZyheedaCommands,
		meshes: Query<(&Self, &mut Visibility, &ChildMeshes), DataOrMeshesChanged<Self>>,
	) where
		TMaterial: Material,
		Self: View<Handle<TMaterial>>,
	{
		for (data, mut visibility, child_meshes) in meshes {
			let material = data.view();

			for entity in child_meshes.iter() {
				commands.try_apply_on(&entity, |mut e| {
					e.try_remove::<MeshMaterial3d<StandardMaterial>>();
					e.try_remove::<MeshMaterial3d<StandardLitMaterial>>();
					e.try_insert(MeshMaterial3d(material.clone()));
				});
			}
			*visibility = Visibility::Visible;
		}
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::components::child_meshes::ChildMeshOf;
	use bevy::render::render_resource::AsBindGroup;
	use testing::{SingleThreadedApp, new_handle};

	#[derive(Component, Debug, PartialEq, Clone)]
	#[require(Visibility)]
	struct _Data(Handle<_Material>);

	impl View<Handle<_Material>> for _Data {
		fn view(&self) -> &'_ Handle<_Material> {
			&self.0
		}
	}

	#[derive(Asset, TypePath, AsBindGroup, Debug, PartialEq, Clone)]
	struct _Material {}

	impl Material for _Material {}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<Assets<_Material>>();
		app.add_systems(Update, _Data::propagate_material::<_Material>);

		app
	}

	#[test]
	fn add_material() {
		let mut app = setup();
		let handle = new_handle();
		let entity = app.world_mut().spawn(_Data(handle.clone())).id();
		let child = app.world_mut().spawn(ChildMeshOf(entity)).id();

		app.update();

		assert_eq!(
			Some(&MeshMaterial3d(handle)),
			app.world().entity(child).get::<MeshMaterial3d<_Material>>(),
		);
	}

	#[test]
	fn remove_standard_material() {
		let mut app = setup();
		let entity = app.world_mut().spawn(_Data(new_handle())).id();
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
		let entity = app.world_mut().spawn(_Data(new_handle())).id();
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
		let entity = app.world_mut().spawn(_Data(new_handle())).id();
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
		let entity = app.world_mut().spawn(_Data(new_handle())).id();
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
		let entity = app.world_mut().spawn(_Data(new_handle())).id();
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
		let entity = app.world_mut().spawn(_Data(new_handle())).id();
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
