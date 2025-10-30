use crate::traits::ray_cast::RayCaster;
use common::traits::{
	cast_ray::TimeOfImpact,
	handles_physics::{Ground, MouseGroundHover, MouseGroundPoint, Raycast},
};

impl Raycast<MouseGroundHover> for RayCaster<'_, '_> {
	fn raycast(&mut self, _: MouseGroundHover) -> Option<MouseGroundPoint> {
		let cam = self.world_cams.single_mut().ok()?;
		let ray = cam.ray?;
		let TimeOfImpact(toi) = self.raycast(Ground { ray })?;

		Some(MouseGroundPoint(ray.origin + ray.direction * toi))
	}
}
