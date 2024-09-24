use crate::{
	components::effect_shader::{EffectShader, EffectShaders},
	traits::get_effect_material::GetEffectMaterial,
};
use bevy::prelude::*;

pub(crate) fn add_child_effect_shader<TEffect: Component + GetEffectMaterial>(
	mut materials: ResMut<Assets<TEffect::TMaterial>>,
	mut effect_shaders: Query<&mut EffectShaders>,
	effects: Query<(Entity, &TEffect), Added<TEffect>>,
	parents: Query<&Parent>,
) {
	for (entity, effect) in &effects {
		for parent in parents.iter_ancestors(entity) {
			let Ok(mut shaders) = effect_shaders.get_mut(parent) else {
				continue;
			};
			let handle = materials.add(effect.get_effect_material());
			shaders.shaders.push(EffectShader::from(handle));
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::effect_shader::EffectShaders;
	use bevy::asset::UntypedAssetId;
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Asset, TypePath)]
	pub struct _Material;

	#[derive(Component)]
	struct _Effect;

	impl GetEffectMaterial for _Effect {
		type TMaterial = _Material;

		fn get_effect_material(&self) -> Self::TMaterial {
			_Material
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<Assets<_Material>>();
		app.add_systems(Update, add_child_effect_shader::<_Effect>);

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
	fn add_child_effect_shader_to_effect_shaders() {
		let mut app = setup();
		let shaders = app.world_mut().spawn(EffectShaders::default()).id();
		app.world_mut().spawn(_Effect).set_parent(shaders);

		app.update();

		let materials = app.world().resource::<Assets<_Material>>();
		let shaders = app.world().entity(shaders).get::<EffectShaders>().unwrap();
		assert_eq!(
			(1, materials_ids(materials)),
			(shaders.shaders.len(), shader_effect_ids(shaders))
		);
	}

	#[test]
	fn add_deep_child_effect_shader_to_effect_shaders() {
		let mut app = setup();
		let shaders = app.world_mut().spawn(EffectShaders::default()).id();
		let child = app.world_mut().spawn_empty().set_parent(shaders).id();
		app.world_mut().spawn(_Effect).set_parent(child);

		app.update();

		let materials = app.world().resource::<Assets<_Material>>();
		let shaders = app.world().entity(shaders).get::<EffectShaders>().unwrap();
		assert_eq!(
			(1, materials_ids(materials)),
			(shaders.shaders.len(), shader_effect_ids(shaders))
		);
	}

	#[test]
	fn add_child_effect_shader_to_effect_shaders_only_once() {
		let mut app = setup();
		let shaders = app.world_mut().spawn(EffectShaders::default()).id();
		app.world_mut().spawn(_Effect).set_parent(shaders);

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
