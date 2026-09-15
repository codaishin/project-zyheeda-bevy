use crate::{
	components::effect_material_data::EffectMaterialData,
	materials::effect_material::EffectMaterial,
	traits::modify_material::ModifyMaterial,
};
use bevy::{ecs::query::QueryFilter, prelude::*};
use common::prelude::*;

impl EffectMaterialData {
	pub(crate) fn modify_material<TPhysics, TEffect>(
		shaders: Query<&Self, TPhysics::TEffectAdded>,
		material_assets: ResMut<Assets<EffectMaterial>>,
	) where
		TPhysics: HandlesPhysicalEffect<TEffect>,
		TEffect: PhysicalEffect + ModifyMaterial + 'static,
	{
		Self::modify_material_internal::<TPhysics::TEffectAdded, TEffect>(shaders, material_assets)
	}

	pub(crate) fn modify_material_internal<TEffectAdded, TEffect>(
		shaders: Query<&Self, TEffectAdded>,
		mut material_assets: ResMut<Assets<EffectMaterial>>,
	) where
		TEffectAdded: QueryFilter,
		TEffect: ModifyMaterial + 'static,
	{
		for Self { material, .. } in shaders {
			let Some(mut material) = material_assets.get_mut(material) else {
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
	use crate::materials::effect_material::EffectFlag;
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
		let mut assets = Assets::default();

		for (id, asset) in materials {
			_ = assets.insert(id, asset);
		}

		app.insert_resource(assets);
		app.add_systems(
			Update,
			EffectMaterialData::modify_material_internal::<With<_Component>, _Effect>,
		);

		app
	}

	#[test]
	fn apply_effect_modification() {
		let first_pass = new_handle();
		let material = new_handle();
		let mut app = setup([(
			&material,
			EffectMaterial::from_first_pass(first_pass.clone()),
		)]);
		app.world_mut().spawn((
			EffectMaterialData {
				material: material.clone(),
				..default()
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
				.get(&material),
		);
	}

	#[test]
	fn do_nothing_when_component_missing() {
		let first_pass = new_handle();
		let material = new_handle();
		let mut app = setup([(
			&material,
			EffectMaterial::from_first_pass(first_pass.clone()),
		)]);
		app.world_mut().spawn(EffectMaterialData {
			material: material.clone(),
			..default()
		});

		app.update();

		assert_eq!(
			Some(&EffectMaterial::from_first_pass(first_pass)),
			app.world()
				.resource::<Assets<EffectMaterial>>()
				.get(&material),
		);
	}
}
