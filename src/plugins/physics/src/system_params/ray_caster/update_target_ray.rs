use crate::system_params::ray_caster::RayCasterMut;
use common::prelude::*;

impl UpdateTargetRay for RayCasterMut<'_, '_> {
	fn update_target_ray(&mut self, ChangedTargetRay(ray): ChangedTargetRay) {
		self.world_camera.ray = ray;
	}
}
