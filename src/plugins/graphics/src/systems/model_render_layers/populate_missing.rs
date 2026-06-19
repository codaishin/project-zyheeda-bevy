use crate::components::model_render_layers::ModelRenderLayers;
use bevy::prelude::*;
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

type NeedsModelRenderLayers = (
	With<Visibility>,
	Without<ModelRenderLayers>,
	Without<ChildOf>,
	Without<Node>,
	Without<Camera>,
	Without<DirectionalLight>,
);

impl ModelRenderLayers {
	pub(crate) fn populate_missing_with(
		layer: impl Into<ModelRenderLayers>,
	) -> impl IntoSystem<(), (), ()> {
		let layer = layer.into();

		IntoSystem::into_system(
			move |mut commands: ZyheedaCommands, roots: Query<Entity, NeedsModelRenderLayers>| {
				for entity in roots {
					commands.try_apply_on(&entity, |mut e| {
						e.try_insert(layer.clone());
					});
				}
			},
		)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::{IsChanged, SingleThreadedApp};

	fn setup(layer: impl Into<ModelRenderLayers>) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			(
				ModelRenderLayers::populate_missing_with(layer),
				IsChanged::<ModelRenderLayers>::detect,
			)
				.chain(),
		);

		app
	}

	#[test]
	fn add_layer_to_visibility() {
		let mut app = setup(22);
		let entity = app.world_mut().spawn(Visibility::default()).id();

		app.update();

		assert_eq!(
			Some(&ModelRenderLayers::from(22)),
			app.world().entity(entity).get::<ModelRenderLayers>(),
		);
	}

	#[test]
	fn add_nothing_if_no_visibility() {
		let mut app = setup(22);
		let entity = app.world_mut().spawn_empty().id();

		app.update();

		assert_eq!(None, app.world().entity(entity).get::<ModelRenderLayers>());
	}

	#[test]
	fn add_nothing_to_children() {
		let mut app = setup(22);
		let parent = app.world_mut().spawn_empty().id();
		let entity = app
			.world_mut()
			.spawn((ChildOf(parent), Visibility::default()))
			.id();

		app.update();

		assert_eq!(None, app.world().entity(entity).get::<ModelRenderLayers>());
	}

	#[test]
	fn add_nothing_to_nodes() {
		let mut app = setup(22);
		let entity = app
			.world_mut()
			.spawn((Node::default(), Visibility::default()))
			.id();

		app.update();

		assert_eq!(None, app.world().entity(entity).get::<ModelRenderLayers>());
	}

	#[test]
	fn add_nothing_to_camera() {
		let mut app = setup(22);
		let entity = app
			.world_mut()
			.spawn((Camera::default(), Visibility::default()))
			.id();

		app.update();

		assert_eq!(None, app.world().entity(entity).get::<ModelRenderLayers>());
	}

	#[test]
	fn add_nothing_to_directional_lights() {
		let mut app = setup(22);
		let entity = app
			.world_mut()
			.spawn((DirectionalLight::default(), Visibility::default()))
			.id();

		app.update();

		assert_eq!(None, app.world().entity(entity).get::<ModelRenderLayers>());
	}

	#[test]
	fn act_only_once() {
		let mut app = setup(22);
		let entity = app.world_mut().spawn(Visibility::default()).id();

		app.update();
		app.update();

		assert_eq!(
			Some(&IsChanged::FALSE),
			app.world()
				.entity(entity)
				.get::<IsChanged<ModelRenderLayers>>(),
		);
	}

	#[test]
	fn act_again_if_pass_layers_missing() {
		let mut app = setup(22);
		let entity = app.world_mut().spawn(Visibility::default()).id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.remove::<ModelRenderLayers>();
		app.update();

		assert_eq!(
			Some(&ModelRenderLayers::from(22)),
			app.world().entity(entity).get::<ModelRenderLayers>(),
		);
	}
}
