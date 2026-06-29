mod camera_transform;
mod render_ui;
mod screen_position;

use crate::resources::camera_parameters::CameraParameters;
use bevy::{
	ecs::system::{SystemParam, SystemParamItem},
	prelude::*,
};
use common::traits::{
	accessors::get::{ContextChanged, GetContext, GetContextMut},
	handles_graphics::CameraHandle,
};

#[derive(SystemParam)]
pub struct CameraParamMut<'w, 's> {
	world_camera: ResMut<'w, CameraParameters>,
	cameras: Query<'w, 's, (&'static GlobalTransform, &'static Camera)>,
}

impl GetContextMut<CameraHandle> for CameraParamMut<'static, 'static> {
	type TContext<'ctx> = CameraContextMut<'ctx>;

	fn get_context_mut<'ctx>(
		param: &'ctx mut SystemParamItem<Self>,
		_: CameraHandle,
	) -> Self::TContext<'ctx> {
		CameraContextMut {
			world_camera: param.world_camera.reborrow(),
			cameras: param.cameras,
		}
	}
}

pub struct CameraContextMut<'ctx> {
	world_camera: Mut<'ctx, CameraParameters>,
	cameras: Query<'ctx, 'ctx, (&'static GlobalTransform, &'static Camera)>,
}

#[derive(SystemParam)]
pub struct CameraParam<'w> {
	world_camera: Res<'w, CameraParameters>,
}

impl GetContext<CameraHandle> for CameraParam<'static> {
	type TContext<'ctx> = CameraContext<'ctx>;

	fn get_context<'ctx>(
		param: &'ctx SystemParamItem<Self>,
		_: CameraHandle,
	) -> Self::TContext<'ctx> {
		CameraContext {
			world_camera: &param.world_camera,
		}
	}
}

pub struct CameraContext<'ctx> {
	world_camera: &'ctx Res<'ctx, CameraParameters>,
}

impl ContextChanged for CameraContext<'_> {
	fn context_changed(&self) -> bool {
		self.world_camera.is_changed()
	}
}
