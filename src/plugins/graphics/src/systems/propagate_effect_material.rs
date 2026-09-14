use crate::{
	components::{child_meshes::ChildMeshes, effect_material_data::EffectMaterialData},
	materials::{effect_material::EffectMaterial, lit_material::StandardLitMaterial},
};
use bevy::prelude::*;
use common::prelude::*;

type DataOrMeshesChanged = Or<(Changed<EffectMaterialData>, Changed<ChildMeshes>)>;

impl EffectMaterialData {
	pub(crate) fn propagate_material(
		mut commands: ZyheedaCommands,
		meshes: Query<(&Self, &mut Visibility, &ChildMeshes), DataOrMeshesChanged>,
		effect_materials: Query<&MeshMaterial3d<EffectMaterial>>,
		mut assets: ResMut<Assets<EffectMaterial>>,
	) {
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
							*effect_material = EffectMaterial::from(data.clone());
							return;
						};
					}

					e.try_insert(MeshMaterial3d(
						assets.add(EffectMaterial::from(data.clone())),
					));
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
	use crate::{
		components::child_meshes::ChildMeshOf,
		materials::effect_material::EffectMaterial,
	};
	use testing::{SingleThreadedApp, new_handle};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<Assets<EffectMaterial>>();
		app.add_systems(Update, EffectMaterialData::propagate_material);

		app
	}

	#[test]
	fn add_material() {
		let first_pass = new_handle();
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(EffectMaterialData::from_first_pass(first_pass.clone()))
			.id();
		let child = app.world_mut().spawn(ChildMeshOf(entity)).id();

		app.update();

		assert!(
			app.world()
				.entity(child)
				.contains::<MeshMaterial3d<EffectMaterial>>(),
		);
	}

	#[test]
	fn set_material_data() {
		let first_pass = new_handle();
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(EffectMaterialData::from_first_pass(first_pass.clone()))
			.id();
		let child = app.world_mut().spawn(ChildMeshOf(entity)).id();

		app.update();

		let MeshMaterial3d(handle) = app
			.world()
			.entity(child)
			.get::<MeshMaterial3d<EffectMaterial>>()
			.unwrap();
		assert_eq!(
			Some(&EffectMaterial::from(EffectMaterialData::from_first_pass(
				first_pass
			))),
			app.world().resource::<Assets<EffectMaterial>>().get(handle),
		);
	}

	#[test]
	fn reuse_material_handle() {
		let first_pass = new_handle();
		let old_handle = new_handle();
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(EffectMaterialData::from_first_pass(first_pass.clone()))
			.id();
		let child = app
			.world_mut()
			.spawn((
				ChildMeshOf(entity),
				MeshMaterial3d::<EffectMaterial>(old_handle.clone()),
			))
			.id();
		_ = app
			.world_mut()
			.resource_mut::<Assets<EffectMaterial>>()
			.insert(&old_handle, EffectMaterial::default());

		app.update();

		let MeshMaterial3d(handle) = app
			.world()
			.entity(child)
			.get::<MeshMaterial3d<EffectMaterial>>()
			.unwrap();
		assert_eq!(
			(
				Some(&EffectMaterial::from(EffectMaterialData::from_first_pass(
					first_pass
				))),
				&old_handle
			),
			(
				app.world()
					.resource::<Assets<EffectMaterial>>()
					.get(&old_handle),
				handle
			)
		);
	}

	#[test]
	fn set_material_data_if_old_handle_invalid() {
		let first_pass = new_handle();
		let old_handle = new_handle();
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(EffectMaterialData::from_first_pass(first_pass.clone()))
			.id();
		let child = app
			.world_mut()
			.spawn((
				ChildMeshOf(entity),
				MeshMaterial3d::<EffectMaterial>(old_handle.clone()),
			))
			.id();

		app.update();

		let MeshMaterial3d(handle) = app
			.world()
			.entity(child)
			.get::<MeshMaterial3d<EffectMaterial>>()
			.unwrap();
		assert_eq!(
			Some(&EffectMaterial::from(EffectMaterialData::from_first_pass(
				first_pass
			))),
			app.world().resource::<Assets<EffectMaterial>>().get(handle),
		);
	}

	#[test]
	fn drop_old_handle_if_invalid() {
		let first_pass = new_handle();
		let old_handle = new_handle();
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(EffectMaterialData::from_first_pass(first_pass.clone()))
			.id();
		let child = app
			.world_mut()
			.spawn((
				ChildMeshOf(entity),
				MeshMaterial3d::<EffectMaterial>(old_handle.clone()),
			))
			.id();

		app.update();

		let MeshMaterial3d(handle) = app
			.world()
			.entity(child)
			.get::<MeshMaterial3d<EffectMaterial>>()
			.unwrap();
		assert_ne!(handle, &old_handle);
	}

	#[test]
	fn remove_standard_material() {
		let mut app = setup();
		let entity = app.world_mut().spawn(EffectMaterialData::default()).id();
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
		let entity = app.world_mut().spawn(EffectMaterialData::default()).id();
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
		let entity = app.world_mut().spawn(EffectMaterialData::default()).id();
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
		let entity = app.world_mut().spawn(EffectMaterialData::default()).id();
		let child = app.world_mut().spawn(ChildMeshOf(entity)).id();

		app.update();
		app.world_mut()
			.entity_mut(child)
			.remove::<MeshMaterial3d<EffectMaterial>>();
		app.update();

		assert_eq!(
			None,
			app.world()
				.entity(child)
				.get::<MeshMaterial3d<EffectMaterial>>(),
		);
	}

	#[test]
	fn act_again_if_children_changed() {
		let first_pass = new_handle();
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(EffectMaterialData::from_first_pass(first_pass))
			.id();
		let child = app.world_mut().spawn(ChildMeshOf(entity)).id();

		app.update();
		app.world_mut()
			.entity_mut(child)
			.remove::<MeshMaterial3d<EffectMaterial>>();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<ChildMeshes>()
			.as_deref_mut();
		app.update();

		assert!(
			app.world()
				.entity(child)
				.contains::<MeshMaterial3d<EffectMaterial>>(),
		);
	}

	#[test]
	fn act_again_if_effect_data_changed() {
		let first_pass = new_handle();
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(EffectMaterialData::from_first_pass(first_pass))
			.id();
		let child = app.world_mut().spawn(ChildMeshOf(entity)).id();

		app.update();
		app.world_mut()
			.entity_mut(child)
			.remove::<MeshMaterial3d<EffectMaterial>>();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<EffectMaterialData>()
			.as_deref_mut();
		app.update();

		assert!(
			app.world()
				.entity(child)
				.contains::<MeshMaterial3d<EffectMaterial>>(),
		);
	}
}
