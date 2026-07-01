use crate::system_params::camera::{CameraContext, CameraContextMut};
use bevy::prelude::*;
use common::traits::handles_graphics::{CameraTransform, CameraTransformMut};

impl CameraTransform for CameraContext<'_> {
	fn camera_transform(&self) -> &Transform {
		&self.world_camera.transform
	}
}

impl CameraTransform for CameraContextMut<'_> {
	fn camera_transform(&self) -> &Transform {
		&self.world_camera.transform
	}
}

impl CameraTransformMut for CameraContextMut<'_> {
	fn camera_transform_mut(&mut self) -> &mut Transform {
		&mut self.world_camera.transform
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		resources::camera_parameters::CameraParameters,
		system_params::camera::CameraParamMut,
	};
	use bevy::{
		app::App,
		ecs::system::{RunSystemError, RunSystemOnce},
	};
	use common::traits::{accessors::get::GetContextMut, handles_graphics::CameraHandle};
	use testing::SingleThreadedApp;

	fn setup(camera: CameraParameters) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(camera);

		app
	}

	mod ctx {
		use super::*;
		use crate::system_params::camera::CameraParam;
		use common::traits::accessors::get::GetContext;

		#[test]
		fn get_transform() -> Result<(), RunSystemError> {
			let mut app = setup(CameraParameters {
				transform: Transform::from_xyz(1., 2., 3.),
				..default()
			});

			let transform = app.world_mut().run_system_once(|c: CameraParam| {
				let ctx = CameraParam::get_context(&c, CameraHandle);
				*ctx.camera_transform()
			})?;

			assert_eq!(Transform::from_xyz(1., 2., 3.), transform);
			Ok(())
		}
	}

	mod ctx_mut {
		use super::*;

		#[test]
		fn get_transform() -> Result<(), RunSystemError> {
			let mut app = setup(CameraParameters {
				transform: Transform::from_xyz(1., 2., 3.),
				..default()
			});

			let transform = app.world_mut().run_system_once(|mut c: CameraParamMut| {
				let ctx = CameraParamMut::get_context_mut(&mut c, CameraHandle);
				*ctx.camera_transform()
			})?;

			assert_eq!(Transform::from_xyz(1., 2., 3.), transform);
			Ok(())
		}

		#[test]
		fn set_transform() -> Result<(), RunSystemError> {
			let mut app = setup(CameraParameters {
				transform: Transform::from_xyz(1., 2., 3.),
				..default()
			});

			let transform = app.world_mut().run_system_once(|mut c: CameraParamMut| {
				let mut ctx = CameraParamMut::get_context_mut(&mut c, CameraHandle);
				*ctx.camera_transform_mut()
			})?;

			assert_eq!(Transform::from_xyz(1., 2., 3.), transform);
			Ok(())
		}
	}
}
