use crate::{
	components::effect_shader::{EffectShader, EffectShaders},
	traits::get_effect_material::GetEffectMaterial,
};
use bevy::prelude::*;

pub(crate) fn add_effect_shader<TEffect: Component + GetEffectMaterial>(
	mut materials: ResMut<Assets<TEffect::TMaterial>>,
	mut effect_shaders: Query<(&mut EffectShaders, &TEffect), Added<TEffect>>,
) {
	for (mut shaders, effect) in &mut effect_shaders {
		let handle = materials.add(effect.get_effect_material());
		shaders.shaders.push(EffectShader::from(handle));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::effect_shader::EffectShaders;
	use bevy::{asset::UntypedAssetId, render::render_resource::AsBindGroup};
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Asset, TypePath, Clone, AsBindGroup)]
	pub struct _Material {}

	impl Material for _Material {}

	#[derive(Component)]
	struct _Effect;

	impl GetEffectMaterial for _Effect {
		type TMaterial = _Material;

		fn get_effect_material(&self) -> Self::TMaterial {
			_Material {}
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<Assets<_Material>>();
		app.add_systems(Update, add_effect_shader::<_Effect>);

		app
	}

	fn materials_ids(materials: &Assets<_Material>) -> Vec<UntypedAssetId> {
		materials
			.iter()
			.map(|(id, _)| id.untyped())
			.collect::<Vec<_>>()
	}

	fn shader_effect_ids(effect_shaders: &EffectShaders) -> Vec<UntypedAssetId> {
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
			.spawn((EffectShaders::default(), _Effect))
			.id();

		app.update();

		let materials = app.world().resource::<Assets<_Material>>();
		let shaders = app.world().entity(shaders).get::<EffectShaders>().unwrap();
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
			.spawn((EffectShaders::default(), _Effect))
			.id();

		app.update();
		app.update();

		let materials = app.world().resource::<Assets<_Material>>();
		let shaders = app.world().entity(shaders).get::<EffectShaders>().unwrap();
		assert_eq!(
			(1, materials_ids(materials)),
			(shaders.shaders.len(), shader_effect_ids(shaders))
		);
	}
}
