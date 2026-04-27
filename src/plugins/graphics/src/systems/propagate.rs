use crate::components::effect_material_config::{EffectShader, EffectShaderMeshes};
use bevy::{camera::visibility::RenderLayers, prelude::*};
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

impl EffectShader {
	pub(crate) fn propagate(layers: impl Into<RenderLayers>) -> impl IntoSystem<(), (), ()> {
		let layers = layers.into();
		IntoSystem::into_system(
			move |mut commands: ZyheedaCommands,
			      shaders: Query<
				(&Self, &mut Visibility, &EffectShaderMeshes),
				Changed<EffectShaderMeshes>,
			>| {
				for (Self { material }, mut visibility, shader_meshes) in shaders {
					for entity in shader_meshes.iter() {
						commands.try_apply_on(&entity, |mut e| {
							e.try_remove::<MeshMaterial3d<StandardMaterial>>();
							e.try_insert((layers.clone(), MeshMaterial3d(material.clone())));
						});
					}
					*visibility = Visibility::Visible;
				}
			},
		)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::effect_material_config::EffectShaderMeshOf,
		materials::effect_material::EffectMaterial,
	};
	use testing::{SingleThreadedApp, new_handle};

	fn setup(layers: impl Into<RenderLayers>) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, EffectShader::propagate(layers));

		app
	}

	#[test]
	fn propagate_material() {
		let material = new_handle();
		let mut app = setup(RenderLayers::default());
		let entity = app
			.world_mut()
			.spawn(EffectShader {
				material: material.clone(),
			})
			.id();
		let child = app.world_mut().spawn(EffectShaderMeshOf(entity)).id();

		app.update();

		assert_eq!(
			Some(&MeshMaterial3d(material)),
			app.world()
				.entity(child)
				.get::<MeshMaterial3d<EffectMaterial>>(),
		);
	}

	#[test]
	fn propagate_render_layer() {
		let mut app = setup(RenderLayers::layer(3));
		let entity = app.world_mut().spawn(EffectShader::default()).id();
		let child = app.world_mut().spawn(EffectShaderMeshOf(entity)).id();

		app.update();

		assert_eq!(
			Some(&RenderLayers::layer(3)),
			app.world().entity(child).get::<RenderLayers>(),
		);
	}

	#[test]
	fn remove_standard_material() {
		let mut app = setup(RenderLayers::default());
		let entity = app.world_mut().spawn(EffectShader::default()).id();
		let child = app
			.world_mut()
			.spawn((
				EffectShaderMeshOf(entity),
				MeshMaterial3d(new_handle::<StandardMaterial>()),
			))
			.id();

		app.update();

		assert_eq!(
			None,
			app.world()
				.entity(child)
				.get::<MeshMaterial3d<StandardMaterial>>(),
		);
	}

	#[test]
	fn set_visibility_to_visible() {
		let mut app = setup(RenderLayers::default());
		let entity = app.world_mut().spawn(EffectShader::default()).id();
		app.world_mut().spawn((
			EffectShaderMeshOf(entity),
			MeshMaterial3d(new_handle::<StandardMaterial>()),
		));

		app.update();

		assert_eq!(
			Some(&Visibility::Visible),
			app.world().entity(entity).get::<Visibility>(),
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup(RenderLayers::default());
		let entity = app.world_mut().spawn(EffectShader::default()).id();
		let child = app.world_mut().spawn(EffectShaderMeshOf(entity)).id();

		app.update();
		app.world_mut()
			.entity_mut(child)
			.remove::<MeshMaterial3d<EffectMaterial>>();
		app.update();

		assert_eq!(
			None,
			app.world()
				.entity(child)
				.get::<MeshMaterial3d<EffectMaterial>>(),
		);
	}

	#[test]
	fn act_only_once_again_if_children_changed() {
		let material = new_handle();
		let mut app = setup(RenderLayers::default());
		let entity = app
			.world_mut()
			.spawn(EffectShader {
				material: material.clone(),
			})
			.id();
		let child = app.world_mut().spawn(EffectShaderMeshOf(entity)).id();

		app.update();
		app.world_mut()
			.entity_mut(child)
			.remove::<MeshMaterial3d<EffectMaterial>>();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<EffectShaderMeshes>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some(&MeshMaterial3d(material)),
			app.world()
				.entity(child)
				.get::<MeshMaterial3d<EffectMaterial>>(),
		);
	}
}
