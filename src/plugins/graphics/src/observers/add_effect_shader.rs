use crate::{
	components::effect_material_handle::EffectMaterialHandle,
	materials::effect_material::EffectMaterial,
	resources::first_pass_image::FirstPassImage,
};
use bevy::prelude::*;
use common::{traits::accessors::get::GetMut, zyheeda_commands::ZyheedaCommands};

impl EffectMaterialHandle {
	pub(crate) fn add_to<TComponent>(
		on_add: On<Add, TComponent>,
		mut materials: ResMut<Assets<EffectMaterial>>,
		mut commands: ZyheedaCommands,
		first_pass_image: Res<FirstPassImage>,
	) where
		TComponent: Component,
	{
		let Some(mut entity) = commands.get_mut(&on_add.entity) else {
			return;
		};

		let material = EffectMaterial::from_first_pass(first_pass_image.handle.clone());
		let material = EffectMaterialHandle {
			material: materials.add(material),
		};

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

		app.insert_resource(FirstPassImage { handle: first_pass });
		app.init_resource::<Assets<EffectMaterial>>();
		app.add_observer(EffectMaterialHandle::add_to::<_Component>);

		app
	}

	#[test]
	fn insert_material() {
		let mut app = setup(new_handle());

		let entity = app.world_mut().spawn(_Component);

		assert!(entity.contains::<EffectMaterialHandle>());
	}

	macro_rules! get_material {
		($app:expr, $entity:expr) => {
			$app.world()
				.entity($entity)
				.get::<EffectMaterialHandle>()
				.and_then(|shader| {
					$app.world()
						.resource::<Assets<EffectMaterial>>()
						.get(&shader.material)
				})
		};
	}

	#[test]
	fn inserted_material_has_first_pass_image() {
		let first_pass = new_handle();
		let mut app = setup(first_pass.clone());

		let entity = app.world_mut().spawn(_Component).id();

		assert_eq!(
			Some(&EffectMaterial::from_first_pass(first_pass)),
			get_material!(app, entity),
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
		entity.remove::<EffectMaterialHandle>();
		entity.insert(_Component);

		assert!(!entity.contains::<EffectMaterialHandle>());
	}
}
