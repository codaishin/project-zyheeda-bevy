use crate::traits::ray_cast::RayCaster;
use bevy::prelude::*;
use common::traits::handles_physics::{Ground, Raycast, TimeOfImpact};

const HORIZONTAL_PLANE: InfinitePlane3d = InfinitePlane3d { normal: Dir3::Y };

impl Raycast<Ground> for RayCaster<'_, '_> {
	fn raycast(&mut self, Ground { ray }: Ground) -> Option<TimeOfImpact> {
		ray.intersect_plane(Vec3::ZERO, HORIZONTAL_PLANE)
			.and_then(|toi| TimeOfImpact::try_from_f32(toi).ok())
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::PhysicsPlugin;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{toi, traits::handles_physics::RaycastSystemParam};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn intersect_origin() -> Result<(), RunSystemError> {
		let mut app = setup();

		let hit = app.world_mut().run_system_once(
			|mut ray_caster: RaycastSystemParam<PhysicsPlugin<()>>| {
				ray_caster.raycast(Ground {
					ray: Ray3d {
						origin: Vec3::Y,
						direction: Dir3::NEG_Y,
					},
				})
			},
		)?;
		assert_eq!(Some(toi!(1.)), hit);
		Ok(())
	}

	#[test]
	fn intersect_off() -> Result<(), RunSystemError> {
		let mut app = setup();

		let hit = app.world_mut().run_system_once(
			|mut ray_caster: RaycastSystemParam<PhysicsPlugin<()>>| {
				ray_caster.raycast(Ground {
					ray: Ray3d {
						origin: Vec3::new(10., 8., 22.),
						direction: Dir3::try_from(Vec3::new(-3., -4., 0.)).unwrap(),
					},
				})
			},
		)?;
		assert_eq!(Some(toi!(10.)), hit);
		Ok(())
	}
}
