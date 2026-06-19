pub(crate) mod populate_missing;
pub(crate) mod propagate;

use crate::components::{
	camera_labels::WorldPass,
	child_meshes::ChildMeshOf,
	model_render_layers::ModelRenderLayers,
};
use bevy::{ecs::system::ScheduleSystem, prelude::*};
use common::systems::link::to_target::LinkToTarget;

type UnlinkedMeshes = (Added<Mesh3d>, Without<ChildMeshOf>);

impl ModelRenderLayers {
	pub(crate) fn systems() -> impl IntoScheduleConfigs<ScheduleSystem, ()> {
		(
			ModelRenderLayers::populate_missing_with(WorldPass),
			UnlinkedMeshes::link_to::<ModelRenderLayers, ChildMeshOf>,
			ModelRenderLayers::propagate_layers,
		)
			.chain()
	}
}

#[cfg(test)]
mod regression_tests {
	use super::*;
	use bevy::camera::visibility::RenderLayers;
	use testing::{SingleThreadedApp, new_handle};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, ModelRenderLayers::systems());

		app
	}

	#[test]
	fn propagate_manually_created_model() {
		let mut app = setup();
		let entity = app.world_mut().spawn(Visibility::Inherited).id();
		let model = app
			.world_mut()
			.spawn((ChildOf(entity), Mesh3d(new_handle())))
			.id();

		app.update();

		let model_layers = ModelRenderLayers::from(WorldPass);
		let render_layers = RenderLayers::from_iter(model_layers.iter().copied());
		assert_eq!(
			(Some(&model_layers), Some(&render_layers)),
			(
				app.world().entity(entity).get::<ModelRenderLayers>(),
				app.world().entity(model).get::<RenderLayers>(),
			)
		);
	}
}
