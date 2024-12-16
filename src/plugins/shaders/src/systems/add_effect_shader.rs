use crate::{
	components::effect_shaders_target::{EffectShaderHandle, EffectShadersTarget},
	traits::get_effect_material::GetEffectMaterial,
};
use bevy::prelude::*;
use common::traits::handles_effect::HandlesEffect;

pub(crate) fn add_effect_shader<TInteractions, TEffect>(
	mut materials: ResMut<Assets<TEffect::TMaterial>>,
	mut effect_shaders: Query<&mut EffectShadersTarget, Added<TInteractions::TEffectComponent>>,
) where
	TInteractions: HandlesEffect<TEffect>,
	TEffect: GetEffectMaterial,
{
	for mut shaders in &mut effect_shaders {
		let handle = materials.add(TEffect::get_effect_material());
		shaders.shaders.insert(EffectShaderHandle::from(handle));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::effect_shaders_target::EffectShadersTarget;
	use bevy::{asset::UntypedAssetId, render::render_resource::AsBindGroup};
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Asset, TypePath, Clone, AsBindGroup)]
	pub struct _Material {}

	impl Material for _Material {}

	struct _Effect;

	#[derive(Component)]
	struct _EffectComponent;

	impl GetEffectMaterial for _Effect {
		type TMaterial = _Material;

		fn get_effect_material() -> Self::TMaterial {
			_Material {}
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

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<Assets<_Material>>();
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
		let mut app = setup();
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
		let mut app = setup();
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
}
