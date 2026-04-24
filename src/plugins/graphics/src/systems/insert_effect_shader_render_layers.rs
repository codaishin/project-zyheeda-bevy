use crate::components::effect_shaders_target::EffectShaderMeshes;
use bevy::{camera::visibility::RenderLayers, prelude::*};
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

pub(crate) fn insert_effect_shader_render_layers<TPass>(
	pass: TPass,
) -> impl Fn(ZyheedaCommands, Query<&EffectShaderMeshes, Changed<EffectShaderMeshes>>)
where
	TPass: Into<RenderLayers>,
{
	let layer = pass.into();

	move |mut commands: ZyheedaCommands,
	      shader_meshes: Query<&EffectShaderMeshes, Changed<EffectShaderMeshes>>| {
		for meshes in shader_meshes {
			for entity in meshes.iter() {
				commands.try_apply_on(&entity, |mut e| {
					e.try_insert(layer.clone());
				});
			}
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::components::effect_shaders_target::EffectShaderMeshOf;
	use bevy::app::{App, Update};
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
		let root = app.world_mut().spawn_empty().id();
		let entities = [
			app.world_mut().spawn(EffectShaderMeshOf(root)).id(),
			app.world_mut().spawn(EffectShaderMeshOf(root)).id(),
			app.world_mut().spawn(EffectShaderMeshOf(root)).id(),
		];

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
		let root = app.world_mut().spawn_empty().id();
		let entities = [
			app.world_mut().spawn(EffectShaderMeshOf(root)).id(),
			app.world_mut().spawn(EffectShaderMeshOf(root)).id(),
			app.world_mut().spawn(EffectShaderMeshOf(root)).id(),
		];

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
		let root = app.world_mut().spawn_empty().id();
		let entities = [
			app.world_mut().spawn(EffectShaderMeshOf(root)).id(),
			app.world_mut().spawn(EffectShaderMeshOf(root)).id(),
			app.world_mut().spawn(EffectShaderMeshOf(root)).id(),
		];

		app.update();
		app.world_mut()
			.entity_mut(root)
			.get_mut::<EffectShaderMeshes>()
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
