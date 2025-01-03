use crate::{
	components::effect_shaders_target::{EffectShaderHandle, EffectShadersTarget},
	traits::get_effect_material::GetEffectMaterial,
};
use bevy::prelude::*;
use common::traits::handles_effect::HandlesEffect;

#[allow(clippy::type_complexity)]
pub(crate) fn add_child_effect_shader<TInteractions, TEffect>(
	mut materials: ResMut<Assets<TEffect::TMaterial>>,
	mut effect_shaders_targets: Query<&mut EffectShadersTarget>,
	effect_shaders: Query<
		Entity,
		(
			Added<TInteractions::TEffectComponent>,
			Without<EffectShadersTarget>,
		),
	>,
	parents: Query<&Parent>,
) where
	TEffect: GetEffectMaterial,
	TInteractions: HandlesEffect<TEffect>,
{
	for entity in &effect_shaders {
		for parent in parents.iter_ancestors(entity) {
			let Ok(mut shaders) = effect_shaders_targets.get_mut(parent) else {
				continue;
			};
			let handle = materials.add(TEffect::get_effect_material());
			shaders.shaders.insert(EffectShaderHandle::from(handle));

			/* This hurts my soul, but we cannot move `effect_shaders_targets` into a lambda for
			 * `find_map` nor mutably borrow `effect_shaders` multiple times, so we iterate and abort
			 * old-school.
			 */
			break;
		}
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
		app.add_systems(Update, add_child_effect_shader::<_HandlesEffects, _Effect>);

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
	fn add_child_effect_shader_to_effect_shaders() {
		let mut app = setup();
		let shaders = app.world_mut().spawn(EffectShadersTarget::default()).id();
		app.world_mut().spawn(_EffectComponent).set_parent(shaders);

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
	fn do_not_add_child_effect_shader_to_effect_shaders_when_child_has_effect_shaders() {
		let mut app = setup();
		let shaders = app.world_mut().spawn(EffectShadersTarget::default()).id();
		app.world_mut()
			.spawn((EffectShadersTarget::default(), _EffectComponent))
			.set_parent(shaders);

		app.update();

		let shaders = app
			.world()
			.entity(shaders)
			.get::<EffectShadersTarget>()
			.unwrap();
		assert_eq!(vec![] as Vec<UntypedAssetId>, shader_effect_ids(shaders));
	}

	#[test]
	fn add_deep_child_effect_shader_to_effect_shaders() {
		let mut app = setup();
		let shaders = app.world_mut().spawn(EffectShadersTarget::default()).id();
		let child = app.world_mut().spawn_empty().set_parent(shaders).id();
		app.world_mut().spawn(_EffectComponent).set_parent(child);

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
	fn add_child_effect_shader_to_effect_shaders_only_once() {
		let mut app = setup();
		let shaders = app.world_mut().spawn(EffectShadersTarget::default()).id();
		app.world_mut().spawn(_EffectComponent).set_parent(shaders);

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
	fn only_add_effect_shader_to_effect_shaders_component_in_closest_parent() {
		let mut app = setup();
		let parent = app.world_mut().spawn(EffectShadersTarget::default()).id();
		let child = app
			.world_mut()
			.spawn(EffectShadersTarget::default())
			.set_parent(parent)
			.id();
		app.world_mut().spawn(_EffectComponent).set_parent(child);

		app.update();

		let materials = app.world().resource::<Assets<_Material>>();
		let parent_shaders = app
			.world()
			.entity(parent)
			.get::<EffectShadersTarget>()
			.unwrap();
		let child_shaders = app
			.world()
			.entity(child)
			.get::<EffectShadersTarget>()
			.unwrap();
		assert_eq!(
			(0, 1, materials_ids(materials)),
			(
				parent_shaders.shaders.len(),
				child_shaders.shaders.len(),
				shader_effect_ids(child_shaders)
			)
		);
	}
}
