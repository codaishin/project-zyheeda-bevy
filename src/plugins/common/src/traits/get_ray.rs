pub mod get_camera_ray;

use bevy::prelude::*;

pub trait GetCamRay<TWindow> {
	fn get_ray(&self, camera_transform: &GlobalTransform, window: &TWindow) -> Option<Ray3d>;
}
