use crate::components::void_sphere::VoidSpherePart;
use bevy::{ecs::system::Query, transform::components::Transform};
use std::ops::Deref;

pub fn ring_rotation(mut agents: Query<(&mut Transform, &VoidSpherePart)>) {
	for (mut transform, part) in &mut agents {
		match part {
			VoidSpherePart::RingA(value) => {
				let value = *value.deref();
				transform.rotate_local_x(value);
				transform.rotate_local_y(value);
			}
			VoidSpherePart::RingB(value) => {
				let value = *value.deref();
				transform.rotate_local_x(value);
				transform.rotate_local_y(value);
				transform.rotate_local_z(value);
			}
			_ => {}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		transform::components::Transform,
	};
	use common::{tools::UnitsPerSecond, traits::clamp_zero_positive::ClampZeroPositive};

	#[test]
	fn rotate_ring_a() {
		let mut app = App::new();
		app.add_systems(Update, ring_rotation);

		let mut transform = Transform::default();
		let ring = app
			.world_mut()
			.spawn((transform, VoidSpherePart::RingA(UnitsPerSecond::new(1.))))
			.id();
		app.update();

		let ring_transform = app.world().entity(ring).get::<Transform>().unwrap();

		transform.rotate_local_x(1.);
		transform.rotate_local_y(1.);

		assert_eq!(transform.rotation, ring_transform.rotation);
	}

	#[test]
	fn rotate_ring_b() {
		let mut app = App::new();
		app.add_systems(Update, ring_rotation);

		let mut transform = Transform::default();
		let ring = app
			.world_mut()
			.spawn((transform, VoidSpherePart::RingB(UnitsPerSecond::new(1.))))
			.id();
		app.update();

		let ring_transform = app.world().entity(ring).get::<Transform>().unwrap();

		transform.rotate_local_x(1.);
		transform.rotate_local_y(1.);
		transform.rotate_local_z(1.);

		assert_eq!(transform.rotation, ring_transform.rotation);
	}
}
