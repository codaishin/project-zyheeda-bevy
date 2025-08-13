use crate::components::effect_shaders_target::EffectShadersTarget;
use bevy::{prelude::*, render::view::RenderLayers};
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

pub(crate) fn insert_effect_shader_render_layers<TPass>(
	pass: TPass,
) -> impl Fn(ZyheedaCommands, Query<&EffectShadersTarget, Changed<EffectShadersTarget>>)
where
	RenderLayers: From<TPass>,
{
	let layer = RenderLayers::from(pass);
	move |mut commands: ZyheedaCommands,
	      shader_targets: Query<&EffectShadersTarget, Changed<EffectShadersTarget>>| {
		for EffectShadersTarget { meshes, .. } in &shader_targets {
			for entity in meshes.iter() {
				commands.try_apply_on(entity, |mut e| {
					e.try_insert(layer.clone());
				});
			}
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::components::effect_shaders_target::EffectShadersTarget;
	use bevy::{
		app::{App, Update},
		utils::default,
	};
	use std::collections::HashSet;
	use testing::SingleThreadedApp;

	fn setup<TPass>(pass: TPass) -> App
	where
		TPass: 'static,
		RenderLayers: From<TPass>,
	{
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, insert_effect_shader_render_layers(pass));

		app
	}

	struct _Pass<const L: usize>;

	impl<const L: usize> From<_Pass<L>> for RenderLayers {
		fn from(_: _Pass<L>) -> Self {
			const { RenderLayers::layer(L) }
		}
	}

	#[test]
	fn insert_render_layer_on_related_entities() {
		let mut app = setup(_Pass::<4>);
		let entities = [
			app.world_mut().spawn_empty().id(),
			app.world_mut().spawn_empty().id(),
			app.world_mut().spawn_empty().id(),
		];
		app.world_mut().spawn(EffectShadersTarget {
			meshes: HashSet::from(entities),
			..default()
		});

		app.update();

		assert_eq!(
			[
				Some(&RenderLayers::layer(4)),
				Some(&RenderLayers::layer(4)),
				Some(&RenderLayers::layer(4)),
			],
			app.world()
				.entity(entities)
				.map(|e| e.get::<RenderLayers>())
		);
	}

	#[test]
	fn insert_render_layer_on_related_entities_only_once() {
		let mut app = setup(_Pass::<4>);
		let entities = [
			app.world_mut().spawn_empty().id(),
			app.world_mut().spawn_empty().id(),
			app.world_mut().spawn_empty().id(),
		];
		app.world_mut().spawn(EffectShadersTarget {
			meshes: HashSet::from(entities),
			..default()
		});

		app.update();
		for entity in entities {
			app.world_mut().entity_mut(entity).remove::<RenderLayers>();
		}
		app.update();

		assert_eq!(
			[None, None, None],
			app.world()
				.entity(entities)
				.map(|e| e.get::<RenderLayers>())
		);
	}

	#[test]
	fn insert_render_layer_on_related_entities_again_when_shader_targets_change() {
		let mut app = setup(_Pass::<4>);
		let entities = [
			app.world_mut().spawn_empty().id(),
			app.world_mut().spawn_empty().id(),
			app.world_mut().spawn_empty().id(),
		];
		let shader_targets = app
			.world_mut()
			.spawn(EffectShadersTarget {
				meshes: HashSet::from(entities),
				..default()
			})
			.id();

		app.update();
		app.world_mut()
			.entity_mut(shader_targets)
			.get_mut::<EffectShadersTarget>()
			.as_deref_mut(); // tell bevy to mark this as changed
		for entity in entities {
			app.world_mut().entity_mut(entity).remove::<RenderLayers>();
		}
		app.update();

		assert_eq!(
			[
				Some(&RenderLayers::layer(4)),
				Some(&RenderLayers::layer(4)),
				Some(&RenderLayers::layer(4)),
			],
			app.world()
				.entity(entities)
				.map(|e| e.get::<RenderLayers>())
		);
	}
}
