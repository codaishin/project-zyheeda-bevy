use crate::system_params::ray_caster::RayCasterMut;
use common::traits::handles_physics::{MouseTerrainHover, MouseTerrainPoint, Raycast, Terrain};

impl Raycast<MouseTerrainHover> for RayCasterMut<'_, '_> {
	fn raycast(&mut self, _: MouseTerrainHover) -> Option<MouseTerrainPoint> {
		let ray = self.world_camera.ray?;
		let toi = self.raycast(Terrain { ray })?;

		Some(MouseTerrainPoint(ray.origin + ray.direction * *toi))
	}
}
