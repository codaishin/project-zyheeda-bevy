use crate::{
	components::{child_meshes::ChildMeshes, effect_material_handle::EffectMaterialHandle},
	materials::lit_material::StandardLitMaterial,
};
use bevy::prelude::*;
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

impl EffectMaterialHandle {
	pub(crate) fn propagate_material(
		mut commands: ZyheedaCommands,
		meshes: Query<(&Self, &mut Visibility, &ChildMeshes), Changed<ChildMeshes>>,
	) {
		for (Self { material }, mut visibility, child_meshes) in meshes {
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
	use super::*;
	use crate::{
		components::child_meshes::ChildMeshOf,
		materials::effect_material::EffectMaterial,
	};
	use testing::{SingleThreadedApp, new_handle};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, EffectMaterialHandle::propagate_material);

		app
	}

	#[test]
	fn propagate_material() {
		let material = new_handle();
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(EffectMaterialHandle {
				material: material.clone(),
			})
			.id();
		let child = app.world_mut().spawn(ChildMeshOf(entity)).id();

		app.update();

		assert_eq!(
			Some(&MeshMaterial3d(material)),
			app.world()
				.entity(child)
				.get::<MeshMaterial3d<EffectMaterial>>(),
		);
	}

	#[test]
	fn remove_standard_material() {
		let mut app = setup();
		let entity = app.world_mut().spawn(EffectMaterialHandle::default()).id();
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
		let entity = app.world_mut().spawn(EffectMaterialHandle::default()).id();
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
		let entity = app.world_mut().spawn(EffectMaterialHandle::default()).id();
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
		let entity = app.world_mut().spawn(EffectMaterialHandle::default()).id();
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
	fn act_only_once_again_if_children_changed() {
		let material = new_handle();
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(EffectMaterialHandle {
				material: material.clone(),
			})
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

		assert_eq!(
			Some(&MeshMaterial3d(material)),
			app.world()
				.entity(child)
				.get::<MeshMaterial3d<EffectMaterial>>(),
		);
	}
}
