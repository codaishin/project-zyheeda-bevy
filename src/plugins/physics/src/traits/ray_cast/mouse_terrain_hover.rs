use crate::traits::ray_cast::RayCaster;
use common::traits::handles_physics::{MouseTerrainHover, MouseTerrainPoint, Raycast, Terrain};

impl Raycast<MouseTerrainHover> for RayCaster<'_, '_> {
	fn raycast(&mut self, _: MouseTerrainHover) -> Option<MouseTerrainPoint> {
		let cam = self.world_cams.single_mut().ok()?;
		let ray = cam.ray?;
		let toi = self.raycast(Terrain { ray })?;

		Some(MouseTerrainPoint(ray.origin + ray.direction * *toi))
	}
}
