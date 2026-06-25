use crate::system_params::ui_camera::UiCameraContextMut;
use bevy::prelude::*;
use common::traits::handles_graphics::RenderUi;

impl RenderUi for UiCameraContextMut<'_> {
	fn render_ui(&mut self, ui: Entity) {
		(self.render_ui)(ui);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{components::camera_labels::UiPass, system_params::ui_camera::UiCameraParamMut};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::traits::{accessors::get::GetContextMut, handles_graphics::CameraHandle};
	use testing::{SingleThreadedApp, assert_count};

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn set_ui_camera_target() -> Result<(), RunSystemError> {
		let mut app = setup();
		let cam = app.world_mut().spawn(UiPass).id();
		let ui = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut p: UiCameraParamMut| {
				let mut ctx = UiCameraParamMut::get_context_mut(&mut p, CameraHandle);
				ctx.render_ui(ui);
			})?;

		assert_eq!(
			Some(&UiTargetCamera(cam)),
			app.world().entity(ui).get::<UiTargetCamera>(),
		);
		Ok(())
	}

	#[test]
	fn set_camera_target_to_new_ui_camera_if_no_ui_cam_exists() -> Result<(), RunSystemError> {
		let mut app = setup();
		let ui = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut p: UiCameraParamMut| {
				let mut ctx = UiCameraParamMut::get_context_mut(&mut p, CameraHandle);
				ctx.render_ui(ui);
			})?;

		let mut cams = app.world_mut().query_filtered::<Entity, With<UiPass>>();
		let [cam] = assert_count!(1, cams.iter(app.world()));
		assert_eq!(
			Some(&UiTargetCamera(cam)),
			app.world().entity(ui).get::<UiTargetCamera>(),
		);
		Ok(())
	}
}
