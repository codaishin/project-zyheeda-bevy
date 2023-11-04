use super::Behavior;
use bevy::prelude::{Ray, Vec3};

pub fn get_move_to(ray: Ray) -> Option<Behavior> {
	let length = ray.intersect_plane(Vec3::ZERO, Vec3::Y)?;
	Some(Behavior::MoveTo(ray.origin + ray.direction * length))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn move_to_zero() {
		let movement = get_move_to(Ray {
			origin: Vec3::Y,
			direction: Vec3::NEG_Y,
		});
		assert_eq!(Some(Behavior::MoveTo(Vec3::ZERO)), movement);
	}

	#[test]
	fn move_to_offset() {
		let movement = get_move_to(Ray {
			origin: Vec3::ONE,
			direction: Vec3::NEG_Y,
		});
		assert_eq!(Some(Behavior::MoveTo(Vec3::new(1., 0., 1.))), movement);
	}

	#[test]
	fn move_to_offset_2() {
		let movement = get_move_to(Ray {
			origin: Vec3::ONE * 2.,
			direction: Vec3::NEG_Y,
		});
		assert_eq!(Some(Behavior::MoveTo(Vec3::new(2., 0., 2.))), movement);
	}
}
