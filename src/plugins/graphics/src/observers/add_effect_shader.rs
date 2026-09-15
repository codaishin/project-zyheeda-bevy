use crate::{
	components::{camera_labels::WorldPass, effect_material_data::EffectMaterialData},
	materials::effect_material::EffectMaterial,
	resources::camera_render_target::CameraRenderTarget,
};
use bevy::prelude::*;
use common::prelude::*;

impl EffectMaterialData {
	pub(crate) fn add_to<TComponent>(
		on_add: On<Add, TComponent>,
		mut commands: ZyheedaCommands,
		first_pass_image: Res<CameraRenderTarget<WorldPass>>,
		mut materials: ResMut<Assets<EffectMaterial>>,
	) where
		TComponent: Component,
	{
		let Some(mut entity) = commands.get_mut(&on_add.entity) else {
			return;
		};

		let material = materials.add(EffectMaterial::from_first_pass(
			first_pass_image.handle.clone(),
		));

		let material = EffectMaterialData {
			material,
			..default()
		};

		entity.try_insert((material, Visibility::Hidden));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::{SingleThreadedApp, assert_some, new_handle};

	#[derive(Component)]
	struct _Component;

	fn setup(first_pass: Handle<Image>) -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<Assets<EffectMaterial>>();
		app.insert_resource(CameraRenderTarget::<WorldPass>::from(first_pass));
		app.add_observer(EffectMaterialData::add_to::<_Component>);

		app
	}

	#[test]
	fn insert_material() {
		let mut app = setup(new_handle());

		let entity = app.world_mut().spawn(_Component);

		assert!(entity.contains::<EffectMaterialData>());
	}

	#[test]
	fn inserted_material_has_first_pass_image() {
		let first_pass = new_handle();
		let mut app = setup(first_pass.clone());

		let entity = app.world_mut().spawn(_Component).id();

		let data = assert_some!(app.world().entity(entity).get::<EffectMaterialData>());
		assert_eq!(
			Some(&EffectMaterial::from_first_pass(first_pass)),
			app.world()
				.resource::<Assets<EffectMaterial>>()
				.get(&data.material),
		);
	}

	#[test]
	fn insert_visibility_hidden() {
		let first_pass = new_handle();
		let mut app = setup(first_pass.clone());

		let entity = app
			.world_mut()
			.spawn((_Component, Visibility::Visible))
			.id();

		assert_eq!(
			Some(&Visibility::Hidden),
			app.world().entity(entity).get::<Visibility>(),
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup(new_handle());

		let mut entity = app.world_mut().spawn(_Component);
		entity.remove::<EffectMaterialData>();
		entity.insert(_Component);

		assert!(!entity.contains::<EffectMaterialData>());
	}
}
