use self::first_pass_image::FirstPassImage;
use crate::{
	components::effect_shaders_target::{EffectShaderHandle, EffectShadersTarget},
	resources::first_pass_image,
	traits::get_effect_material::GetEffectMaterial,
};
use bevy::prelude::*;
use common::traits::handles_effects::{Effect, HandlesEffect};

#[allow(clippy::type_complexity)]
pub(crate) fn add_child_effect_shader<TInteractions, TEffect>(
	mut materials: ResMut<Assets<TEffect::TMaterial>>,
	mut effect_shaders_targets: Query<&mut EffectShadersTarget>,
	first_pass_image: Res<FirstPassImage>,
	effect_shaders: Query<
		Entity,
		(
			Added<TInteractions::TEffectComponent>,
			Without<EffectShadersTarget>,
		),
	>,
	children: Query<&ChildOf>,
) where
	TEffect: GetEffectMaterial + Effect,
	TInteractions: HandlesEffect<TEffect>,
{
	for entity in &effect_shaders {
		for parent in children.iter_ancestors(entity) {
			let Ok(mut shaders) = effect_shaders_targets.get_mut(parent) else {
				continue;
			};
			let handle = materials.add(TEffect::get_effect_material(&first_pass_image.handle));
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
	use testing::{SingleThreadedApp, new_handle};

	#[derive(Asset, TypePath, Clone, AsBindGroup, Debug, PartialEq)]
	pub struct _Material {
		first_pass: Handle<Image>,
	}

	impl Material for _Material {}

	struct _Effect;

	impl Effect for _Effect {
		type TTarget = ();
	}

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
		type TEffectComponent = _EffectComponent;

		fn effect(_: _Effect) -> Self::TEffectComponent {
			_EffectComponent
		}

		fn attribute(_: ()) -> impl Bundle {}
	}

	fn setup(first_pass: Handle<Image>) -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<Assets<_Material>>();
		app.insert_resource(FirstPassImage { handle: first_pass });
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
		let mut app = setup(new_handle::<Image>());
		let shaders = app.world_mut().spawn(EffectShadersTarget::default()).id();
		app.world_mut()
			.spawn(_EffectComponent)
			.insert(ChildOf(shaders));

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
		let mut app = setup(new_handle::<Image>());
		let shaders = app.world_mut().spawn(EffectShadersTarget::default()).id();
		app.world_mut()
			.spawn((EffectShadersTarget::default(), _EffectComponent))
			.insert(ChildOf(shaders));

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
		let mut app = setup(new_handle::<Image>());
		let shaders = app.world_mut().spawn(EffectShadersTarget::default()).id();
		let child = app.world_mut().spawn_empty().insert(ChildOf(shaders)).id();
		app.world_mut()
			.spawn(_EffectComponent)
			.insert(ChildOf(child));

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
		let mut app = setup(new_handle::<Image>());
		let shaders = app.world_mut().spawn(EffectShadersTarget::default()).id();
		app.world_mut()
			.spawn(_EffectComponent)
			.insert(ChildOf(shaders));

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
		let mut app = setup(new_handle::<Image>());
		let parent = app.world_mut().spawn(EffectShadersTarget::default()).id();
		let child = app
			.world_mut()
			.spawn(EffectShadersTarget::default())
			.insert(ChildOf(parent))
			.id();
		app.world_mut()
			.spawn(_EffectComponent)
			.insert(ChildOf(child));

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

	#[test]
	fn assign_first_pass_image_handle() {
		let first_pass = new_handle::<Image>();
		let mut app = setup(first_pass.clone());
		let shaders = app.world_mut().spawn(EffectShadersTarget::default()).id();
		app.world_mut()
			.spawn(_EffectComponent)
			.insert(ChildOf(shaders));

		app.update();

		let materials = app.world().resource::<Assets<_Material>>();
		let material = materials.get(materials.ids().next().unwrap());
		assert_eq!(Some(&_Material { first_pass }), material);
	}
}
