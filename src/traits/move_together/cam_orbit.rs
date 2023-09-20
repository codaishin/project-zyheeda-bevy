use super::MoveTogether;
use crate::components::CamOrbit;
use bevy::prelude::{Transform, Vec3};

impl MoveTogether for CamOrbit {
	fn move_together_with(&mut self, transform: &mut Transform, new_position: Vec3) {
		let offset = (transform.translation - self.center).normalize() * self.distance;

		self.center = new_position;
		transform.translation = self.center + offset;
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn set_partner_to_proper_distance() {
		let mut orbit = CamOrbit {
			center: Vec3::ZERO,
			distance: 42.,
			sensitivity: 0.,
		};
		let mut transform = Transform::from_xyz(1., 0., 0.);

		orbit.move_together_with(&mut transform, orbit.center);

		assert_eq!(Vec3::new(42., 0., 0.), transform.translation);
	}

	#[test]
	fn set_partner_to_proper_distance_from_offset() {
		let mut orbit = CamOrbit {
			center: Vec3::ONE,
			distance: 11.,
			sensitivity: 0.,
		};
		let mut transform = Transform::from_xyz(1., 1., 7.);

		orbit.move_together_with(&mut transform, orbit.center);

		assert_eq!(Vec3::new(1., 1., 12.), transform.translation);
	}

	#[test]
	fn set_center_to_new_position() {
		let mut orbit = CamOrbit {
			center: Vec3::ZERO,
			distance: 42.,
			sensitivity: 0.,
		};
		let mut transform = Transform::from_xyz(1., 0., 0.);

		orbit.move_together_with(&mut transform, Vec3::new(11., -4., 9.));

		assert_eq!(Vec3::new(11., -4., 9.), orbit.center);
	}

	#[test]
	fn also_move_partner_transform() {
		let mut orbit = CamOrbit {
			center: Vec3::ZERO,
			distance: 10.,
			sensitivity: 0.,
		};
		let mut transform = Transform::from_xyz(1., 0., 0.);

		orbit.move_together_with(&mut transform, Vec3::new(1., 2., 3.));

		assert_eq!(Vec3::new(11., 2., 3.), transform.translation);
	}
}
