use crate::{
	components::effect_shaders_target::{EffectShaderHandle, EffectShadersTarget},
	resources::first_pass_image::FirstPassImage,
	traits::get_effect_material::GetEffectMaterial,
};
use bevy::prelude::*;
use common::traits::handles_effect::HandlesEffect;

pub(crate) fn add_effect_shader<TInteractions, TEffect>(
	mut materials: ResMut<Assets<TEffect::TMaterial>>,
	mut effect_shaders: Query<&mut EffectShadersTarget, Added<TInteractions::TEffectComponent>>,
	first_pass_image: Res<FirstPassImage>,
) where
	TInteractions: HandlesEffect<TEffect>,
	TEffect: GetEffectMaterial,
{
	for mut shaders in &mut effect_shaders {
		let handle = materials.add(TEffect::get_effect_material(&first_pass_image.handle));
		shaders.shaders.insert(EffectShaderHandle::from(handle));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::effect_shaders_target::EffectShadersTarget;
	use bevy::{asset::UntypedAssetId, render::render_resource::AsBindGroup};
	use common::test_tools::utils::{new_handle, SingleThreadedApp};

	#[derive(Asset, TypePath, Clone, AsBindGroup, Debug, PartialEq)]
	pub struct _Material {
		first_pass: Handle<Image>,
	}

	impl Material for _Material {}

	struct _Effect;

	#[derive(Component)]
	struct _EffectComponent;

	impl GetEffectMaterial for _Effect {
		type TMaterial = _Material;

		fn get_effect_material(first_pass: &Handle<Image>) -> Self::TMaterial {
			_Material {
				first_pass: first_pass.clone(),
			}
		}
	}

	struct _HandlesEffects;

	impl HandlesEffect<_Effect> for _HandlesEffects {
		type TTarget = ();
		type TEffectComponent = _EffectComponent;

		fn effect(_: _Effect) -> Self::TEffectComponent {
			_EffectComponent
		}

		fn attribute(_: Self::TTarget) -> impl Bundle {}
	}

	fn setup(first_pass: Handle<Image>) -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<Assets<_Material>>();
		app.insert_resource(FirstPassImage { handle: first_pass });
		app.add_systems(Update, add_effect_shader::<_HandlesEffects, _Effect>);

		app
	}

	fn materials_ids(materials: &Assets<_Material>) -> Vec<UntypedAssetId> {
		materials
			.iter()
			.map(|(id, _)| id.untyped())
			.collect::<Vec<_>>()
	}

	fn shader_effect_ids(effect_shaders: &EffectShadersTarget) -> Vec<UntypedAssetId> {
		effect_shaders
			.shaders
			.iter()
			.map(|h| h.id())
			.collect::<Vec<_>>()
	}

	#[test]
	fn add_material_handle_to_effect_shaders() {
		let mut app = setup(new_handle::<Image>());
		let shaders = app
			.world_mut()
			.spawn((EffectShadersTarget::default(), _EffectComponent))
			.id();

		app.update();

		let materials = app.world().resource::<Assets<_Material>>();
		let shaders = app
			.world()
			.entity(shaders)
			.get::<EffectShadersTarget>()
			.unwrap();
		assert_eq!(
			(1, materials_ids(materials)),
			(shaders.shaders.len(), shader_effect_ids(shaders))
		);
	}

	#[test]
	fn add_material_handle_to_effect_shaders_only_once() {
		let mut app = setup(new_handle::<Image>());
		let shaders = app
			.world_mut()
			.spawn((EffectShadersTarget::default(), _EffectComponent))
			.id();

		app.update();
		app.update();

		let materials = app.world().resource::<Assets<_Material>>();
		let shaders = app
			.world()
			.entity(shaders)
			.get::<EffectShadersTarget>()
			.unwrap();
		assert_eq!(
			(1, materials_ids(materials)),
			(shaders.shaders.len(), shader_effect_ids(shaders))
		);
	}

	#[test]
	fn assign_first_pass_image_handle() {
		let first_pass = new_handle::<Image>();
		let mut app = setup(first_pass.clone());
		app.world_mut()
			.spawn((EffectShadersTarget::default(), _EffectComponent));

		app.update();

		let materials = app.world().resource::<Assets<_Material>>();
		let material = materials.get(materials.ids().next().unwrap());
		assert_eq!(Some(&_Material { first_pass }), material);
	}
}
