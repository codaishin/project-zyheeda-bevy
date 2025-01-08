use bevy::math::Vec3;

pub trait IntersectAt {
	fn intersect_at(&self, height: f32) -> Option<Vec3>;
}
