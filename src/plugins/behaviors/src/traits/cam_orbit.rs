use super::{MoveTogether, Orbit, Vec2Radians};
use crate::components::CamOrbit;
use bevy::prelude::*;

impl Orbit for CamOrbit {
	fn orbit(&self, agent: &mut Transform, angles: Vec2Radians) {
		let mut arm =
			Transform::from_translation(self.center).looking_at(agent.translation, Vec3::Y);
		let angles = angles * self.sensitivity;

		arm.rotate_y(-angles.x);
		arm.rotate_local_x(angles.y);

		agent.translation = self.center + (arm.forward() * self.distance);
		agent.look_at(self.center, Vec3::Y);
	}
}

impl MoveTogether for CamOrbit {
	fn move_together_with(&mut self, transform: &mut Transform, new_position: Vec3) {
		let offset = (transform.translation - self.center).normalize() * self.distance;

		self.center = new_position;
		transform.translation = self.center + offset;
	}
}

#[cfg(test)]
mod test_orbit {
	use super::*;
	use common::test_tools::utils::assert_eq_approx;
	use std::f32::consts::PI;

	#[test]
	fn apply_distance() {
		let mut agent = Transform::from_translation(Vec3::X);
		let cam_orbit = CamOrbit {
			center: Vec3::ZERO,
			distance: 11.,
			sensitivity: 1.,
		};

		cam_orbit.orbit(&mut agent, Vec2Radians::ZERO);

		assert_eq_approx!(Vec3::new(11., 0., 0.), agent.translation, 0.00001);
	}

	#[test]
	fn apply_distance_with_center_offset() {
		let mut agent = Transform::from_translation(Vec3::new(1., 2., 1.));
		let cam_orbit = CamOrbit {
			center: Vec3::ONE,
			distance: 11.,
			sensitivity: 1.,
		};

		cam_orbit.orbit(&mut agent, Vec2Radians::ZERO);

		assert_eq_approx!(Vec3::new(1., 12., 1.), agent.translation, 0.00001);
	}

	#[test]
	fn rotate_around_y() {
		let cam_orbit = CamOrbit {
			center: Vec3::new(0., 2., 0.),
			distance: 3.,
			sensitivity: 1.,
		};
		let mut agent = Transform::from_xyz(3., 2., 0.);
		let quarter_circle = PI / 2.;

		cam_orbit.orbit(&mut agent, Vec2Radians::new(quarter_circle, 0.));

		assert_eq_approx!(Vec3::new(0., 2., 3.), agent.translation, 0.00001);
	}

	#[test]
	fn rotate_around_x() {
		let cam_orbit = CamOrbit {
			center: Vec3::new(0., 2., 0.),
			distance: 3.,
			sensitivity: 1.,
		};
		let mut agent = Transform::from_xyz(3., 2., 0.);
		let quarter_circle = PI / 2.;

		cam_orbit.orbit(&mut agent, Vec2Radians::new(0., quarter_circle));

		assert_eq_approx!(Vec3::new(0., 5., 0.), agent.translation, 0.00001);
	}

	#[test]
	fn rotate_with_sensitivity() {
		let cam_orbit = CamOrbit {
			center: Vec3::ZERO,
			distance: 1.,
			sensitivity: 0.5,
		};
		let mut agent = Transform::from_translation(Vec3::X);
		let half_circle = PI;

		cam_orbit.orbit(&mut agent, Vec2Radians::new(half_circle, half_circle));

		assert_eq_approx!(Vec3::Y, agent.translation, 0.00001);
	}

	#[test]
	fn look_at_center() {
		let cam_orbit = CamOrbit {
			center: Vec3::new(1., 2., 3.),
			distance: 3.,
			sensitivity: 1.,
		};
		let mut agent = Transform::from_xyz(1., 1., 1.);

		cam_orbit.orbit(&mut agent, Vec2Radians::ZERO);

		let expected_forward = (cam_orbit.center - agent.translation).normalize();

		assert_eq_approx!(expected_forward, agent.forward(), 0.00001);
	}
}

#[cfg(test)]
mod test_move_together {
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
