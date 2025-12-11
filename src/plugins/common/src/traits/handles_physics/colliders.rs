use bevy::prelude::*;

pub trait Colliders {
	type TCollider: Component + From<Collider>;
}

#[derive(Debug, PartialEq)]
pub struct Collider {
	pub shape: Shape,
	pub collider_type: ColliderType,
	pub center_offset: Vec3,
	pub rotation: Quat,
}

impl Collider {
	pub const fn from_shape(shape: Shape) -> Self {
		Self {
			shape,
			collider_type: ColliderType::Terrain,
			center_offset: Vec3::ZERO,
			rotation: Quat::IDENTITY,
		}
	}

	pub const fn with_center_offset(mut self, center_offset: Vec3) -> Self {
		self.center_offset = center_offset;
		self
	}

	pub const fn with_rotation(mut self, rotation: Quat) -> Self {
		self.rotation = rotation;
		self
	}

	pub const fn with_collision_type(mut self, collider_type: ColliderType) -> Self {
		self.collider_type = collider_type;
		self
	}
}

impl From<Shape> for Collider {
	fn from(shape: Shape) -> Self {
		Self::from_shape(shape)
	}
}

/// Shape of a collider
///
/// Coordinates are in line with the bevy coordinate system.
/// So without rotations they are:
/// - `x`: width
/// - `y`: height
/// - `z`: depth
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Shape {
	Sphere {
		radius: f32,
	},
	Cuboid {
		half_x: f32,
		half_y: f32,
		half_z: f32,
	},
	Capsule {
		half_y: f32,
		radius: f32,
	},
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ColliderType {
	Agent,
	Terrain,
}
