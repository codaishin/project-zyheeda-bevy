use crate::{
	components::effect_material_config::EffectShader,
	materials::effect_material::EffectMaterial,
	traits::modify_material::ModifyMaterial,
};
use bevy::prelude::*;
use common::traits::handles_physics::{Effect, HandlesPhysicalEffect};

impl EffectShader {
	pub(crate) fn modify_material<TPhysics, TEffect>(
		shaders: Query<&Self, Added<TPhysics::TEffectComponent>>,
		materials: ResMut<Assets<EffectMaterial>>,
	) where
		TPhysics: HandlesPhysicalEffect<TEffect>,
		TEffect: Effect + ModifyMaterial + 'static,
	{
		Self::modify_material_internal::<TPhysics::TEffectComponent, TEffect>(shaders, materials)
	}

	pub(crate) fn modify_material_internal<TEffectComponent, TEffect>(
		shaders: Query<&Self, Added<TEffectComponent>>,
		mut materials: ResMut<Assets<EffectMaterial>>,
	) where
		TEffectComponent: Component,
		TEffect: ModifyMaterial + 'static,
	{
		for EffectShader { material } in shaders {
			let Some(material) = materials.get_mut(material) else {
				continue;
			};

			TEffect::modify_material(material);
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
			EffectShader::modify_material_internal::<_Component, _Effect>,
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
			EffectShader {
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
		app.world_mut().spawn(EffectShader {
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

	#[test]
	fn act_only_once() {
		let first_pass = new_handle();
		let handle = new_handle();
		let material = EffectMaterial::from_first_pass(first_pass.clone());
		let mut app = setup([(&handle, material)]);
		app.world_mut().spawn((
			EffectShader {
				material: handle.clone(),
			},
			_Component,
		));

		app.update();
		let mut materials = app.world_mut().resource_mut::<Assets<EffectMaterial>>();
		*materials.get_mut(&handle).unwrap() = EffectMaterial::from_first_pass(first_pass.clone());
		app.update();

		assert_eq!(
			Some(&EffectMaterial::from_first_pass(first_pass)),
			app.world()
				.resource::<Assets<EffectMaterial>>()
				.get(&handle),
		);
	}
}
