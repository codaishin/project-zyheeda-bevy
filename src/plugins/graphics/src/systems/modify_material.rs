use crate::{
	components::effect_material_handle::EffectMaterialHandle,
	materials::effect_material::EffectMaterial,
	traits::modify_material::ModifyMaterial,
};
use bevy::{ecs::query::QueryFilter, prelude::*};
use common::prelude::*;

impl EffectMaterialHandle {
	pub(crate) fn modify_material<TPhysics, TEffect>(
		shaders: Query<&Self, TPhysics::TEffectAdded>,
		materials: ResMut<Assets<EffectMaterial>>,
	) where
		TPhysics: HandlesPhysicalEffect<TEffect>,
		TEffect: PhysicalEffect + ModifyMaterial + 'static,
	{
		Self::modify_material_internal::<TPhysics::TEffectAdded, TEffect>(shaders, materials)
	}

	pub(crate) fn modify_material_internal<TEffectAdded, TEffect>(
		shaders: Query<&Self, TEffectAdded>,
		mut materials: ResMut<Assets<EffectMaterial>>,
	) where
		TEffectAdded: QueryFilter,
		TEffect: ModifyMaterial + 'static,
	{
		for EffectMaterialHandle { material } in shaders {
			let Some(mut material) = materials.get_mut(material) else {
				continue;
			};

			TEffect::modify_material(&mut material);
		}
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::materials::effect_material::{EffectFlag, EffectMaterial};
	use bevy::color::palettes::tailwind::CYAN_300;
	use testing::{SingleThreadedApp, new_handle};

	#[derive(Component)]
	struct _Component;

	struct _Effect;

	impl ModifyMaterial for _Effect {
		fn modify_material(material: &mut EffectMaterial) {
			material.add_flag(EffectFlag::Fresnel(CYAN_300.into()));
		}
	}

	fn setup<const N: usize>(materials: [(&Handle<EffectMaterial>, EffectMaterial); N]) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut material_assets = Assets::default();

		for (id, asset) in materials {
			_ = material_assets.insert(id, asset);
		}

		app.insert_resource(material_assets);
		app.add_systems(
			Update,
			EffectMaterialHandle::modify_material_internal::<With<_Component>, _Effect>,
		);

		app
	}

	#[test]
	fn apply_effect_modification() {
		let first_pass = new_handle();
		let handle = new_handle();
		let material = EffectMaterial::from_first_pass(first_pass.clone());
		let mut app = setup([(&handle, material)]);
		app.world_mut().spawn((
			EffectMaterialHandle {
				material: handle.clone(),
			},
			_Component,
		));

		app.update();

		let mut expected = EffectMaterial::from_first_pass(first_pass);
		_Effect::modify_material(&mut expected);
		assert_eq!(
			Some(&expected),
			app.world()
				.resource::<Assets<EffectMaterial>>()
				.get(&handle)
		);
	}

	#[test]
	fn do_nothing_when_component_missing() {
		let first_pass = new_handle();
		let handle = new_handle();
		let material = EffectMaterial::from_first_pass(first_pass.clone());
		let mut app = setup([(&handle, material)]);
		app.world_mut().spawn(EffectMaterialHandle {
			material: handle.clone(),
		});

		app.update();

		assert_eq!(
			Some(&EffectMaterial::from_first_pass(first_pass)),
			app.world()
				.resource::<Assets<EffectMaterial>>()
				.get(&handle)
		);
	}
}
