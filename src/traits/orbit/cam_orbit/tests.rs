use std::f32::consts::PI;

use crate::test_tools::assert_eq_approx;

use super::*;

#[test]
fn apply_distance() {
	let mut agent = Transform::from_translation(Vec3::X);
	let cam_orbit = CamOrbit {
		center: Vec3::ZERO,
		distance: 11.,
		sensitivity: 1.,
	};

	cam_orbit.orbit(&mut agent, Vec2Radians::ZERO);

	assert_eq_approx(Vec3::new(11., 0., 0.), agent.translation, 0.00001);
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

	assert_eq_approx(Vec3::new(1., 12., 1.), agent.translation, 0.00001);
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

	assert_eq_approx(Vec3::new(0., 2., 3.), agent.translation, 0.00001);
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

	assert_eq_approx(Vec3::new(0., 5., 0.), agent.translation, 0.00001);
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

	assert_eq_approx(Vec3::Y, agent.translation, 0.00001);
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

	assert_eq_approx(expected_forward, agent.forward(), 0.00001);
}
