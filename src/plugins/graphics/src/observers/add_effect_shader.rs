use crate::{
	components::{camera_labels::WorldPass, effect_material_data::EffectMaterialData},
	resources::camera_render_target::CameraRenderTarget,
};
use bevy::prelude::*;
use common::prelude::*;

impl EffectMaterialData {
	pub(crate) fn add_to<TComponent>(
		on_add: On<Add, TComponent>,
		mut commands: ZyheedaCommands,
		first_pass_image: Res<CameraRenderTarget<WorldPass>>,
	) where
		TComponent: Component,
	{
		let Some(mut entity) = commands.get_mut(&on_add.entity) else {
			return;
		};

		let material = EffectMaterialData::from_first_pass(first_pass_image.handle.clone());

		entity.try_insert((material, Visibility::Hidden));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::{SingleThreadedApp, new_handle};

	#[derive(Component)]
	struct _Component;

	fn setup(first_pass: Handle<Image>) -> App {
		let mut app = App::new().single_threaded(Update);

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

		assert_eq!(
			Some(&EffectMaterialData::from_first_pass(first_pass)),
			app.world().entity(entity).get::<EffectMaterialData>(),
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
