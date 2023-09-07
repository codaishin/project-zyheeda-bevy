use bevy::prelude::*;

pub fn assert_eq_approx(left: Vec3, right: Vec3, tolerance: f32) {
	assert!(
		left.abs_diff_eq(right, tolerance),
		"left {} was not approximately equal to right {}",
		left,
		right
	);
}
