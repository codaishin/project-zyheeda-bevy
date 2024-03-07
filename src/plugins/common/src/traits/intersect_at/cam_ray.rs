use super::IntersectAt;
use crate::resources::CamRay;
use bevy::math::{primitives::Plane3d, Vec3};

impl IntersectAt for CamRay {
	fn intersect_at(&self, height: f32) -> Option<Vec3> {
		let ray = self.0?;
		let toi = ray.intersect_plane(Vec3::new(0., height, 0.), Plane3d::new(Vec3::Y))?;

		Some(ray.origin + ray.direction * toi)
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use bevy::math::Ray3d;

	#[test]
	fn intersect_at_zero_elevation() {
		let ray = CamRay(Some(Ray3d {
			origin: Vec3::new(1., 4., 0.),
			direction: Vec3::new(-3., -4., 0.).normalize().try_into().unwrap(),
		}));

		assert_eq!(Some(Vec3::new(-2., 0., 0.)), ray.intersect_at(0.));
	}

	#[test]
	fn intersect_at_some_elevation() {
		let ray = CamRay(Some(Ray3d {
			origin: Vec3::new(1., 5., 0.),
			direction: Vec3::new(-3., -4., 0.).normalize().try_into().unwrap(),
		}));

		assert_eq!(Some(Vec3::new(-2., 1., 0.)), ray.intersect_at(1.));
	}
}
