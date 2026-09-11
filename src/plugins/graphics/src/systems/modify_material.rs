use crate::{
	components::effect_material_data::EffectMaterialData,
	traits::modify_material::ModifyMaterial,
};
use bevy::{ecs::query::QueryFilter, prelude::*};
use common::prelude::*;

impl EffectMaterialData {
	pub(crate) fn modify_material<TPhysics, TEffect>(
		shaders: Query<&mut Self, TPhysics::TEffectAdded>,
	) where
		TPhysics: HandlesPhysicalEffect<TEffect>,
		TEffect: PhysicalEffect + ModifyMaterial + 'static,
	{
		Self::modify_material_internal::<TPhysics::TEffectAdded, TEffect>(shaders)
	}

	pub(crate) fn modify_material_internal<TEffectAdded, TEffect>(
		shaders: Query<&mut Self, TEffectAdded>,
	) where
		TEffectAdded: QueryFilter,
		TEffect: ModifyMaterial + 'static,
	{
		for mut material in shaders {
			TEffect::modify_material(&mut material);
		}
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::components::effect_material_data::EffectFlag;
	use bevy::color::palettes::tailwind::CYAN_300;
	use testing::{SingleThreadedApp, new_handle};

	#[derive(Component)]
	struct _Component;

	struct _Effect;

	impl ModifyMaterial for _Effect {
		fn modify_material(material: &mut EffectMaterialData) {
			material.add_flag(EffectFlag::Fresnel(CYAN_300.into()));
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			EffectMaterialData::modify_material_internal::<With<_Component>, _Effect>,
		);

		app
	}

	#[test]
	fn apply_effect_modification() {
		let first_pass = new_handle();
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				EffectMaterialData::from_first_pass(first_pass.clone()),
				_Component,
			))
			.id();

		app.update();

		let mut expected = EffectMaterialData::from_first_pass(first_pass);
		_Effect::modify_material(&mut expected);
		assert_eq!(
			Some(&expected),
			app.world().entity(entity).get::<EffectMaterialData>(),
		);
	}

	#[test]
	fn do_nothing_when_component_missing() {
		let first_pass = new_handle();
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(EffectMaterialData::from_first_pass(first_pass.clone()))
			.id();

		app.update();

		assert_eq!(
			Some(&EffectMaterialData::from_first_pass(first_pass)),
			app.world().entity(entity).get::<EffectMaterialData>(),
		);
	}
}
