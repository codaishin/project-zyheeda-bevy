use crate::traits::ray_cast::RayCaster;
use bevy::prelude::*;
use common::traits::{
	cast_ray::TimeOfImpact,
	handles_physics::{Ground, Raycast},
};

const HORIZONTAL_PLANE: InfinitePlane3d = InfinitePlane3d { normal: Dir3::Y };

impl Raycast<Ground> for RayCaster<'_, '_> {
	fn raycast(&self, ray: Ray3d, _: Ground) -> Option<TimeOfImpact> {
		ray.intersect_plane(Vec3::ZERO, HORIZONTAL_PLANE)
			.map(TimeOfImpact)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn intersect_origin() -> Result<(), RunSystemError> {
		let mut app = setup();

		let hit = app.world_mut().run_system_once(|ray_caster: RayCaster| {
			ray_caster.raycast(
				Ray3d {
					origin: Vec3::Y,
					direction: Dir3::NEG_Y,
				},
				Ground,
			)
		})?;
		assert_eq!(Some(TimeOfImpact(1.)), hit);
		Ok(())
	}

	#[test]
	fn intersect_off() -> Result<(), RunSystemError> {
		let mut app = setup();

		let hit = app.world_mut().run_system_once(|ray_caster: RayCaster| {
			ray_caster.raycast(
				Ray3d {
					origin: Vec3::new(10., 8., 22.),
					direction: Dir3::try_from(Vec3::new(-3., -4., 0.)).unwrap(),
				},
				Ground,
			)
		})?;
		assert_eq!(Some(TimeOfImpact(10.)), hit);
		Ok(())
	}
}
