use crate::system_params::camera::CameraContextMut;
use bevy::prelude::*;
use common::traits::handles_graphics::RenderUi;

impl RenderUi for CameraContextMut<'_> {
	fn render_ui(&mut self, ui: Entity) {
		self.world_camera.uis.push(ui);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		resources::camera_parameters::CameraParameters,
		system_params::camera::CameraParamMut,
	};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::traits::{accessors::get::GetContextMut, handles_graphics::CameraHandle};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<CameraParameters>();

		app
	}

	#[test]
	fn set_ui_camera_target() -> Result<(), RunSystemError> {
		let mut app = setup();
		let ui = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut p: CameraParamMut| {
				let mut ctx = CameraParamMut::get_context_mut(&mut p, CameraHandle);
				ctx.render_ui(ui);
			})?;

		assert_eq!(vec![ui], app.world().resource::<CameraParameters>().uis);
		Ok(())
	}
}
