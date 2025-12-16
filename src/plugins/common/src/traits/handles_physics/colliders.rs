use crate::{
	tools::Units,
	traits::iteration::{Iter, IterFinite},
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub trait HandlesColliders {
	type TCollider: Component + From<Collider>;
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Collider {
	pub shape: Shape,
	pub collider_type: ColliderType,
	pub blocker_types: HashSet<Blocker>,
	pub center_offset: Vec3,
	pub rotation: Quat,
}

impl Collider {
	pub fn from_shape(shape: Shape) -> Self {
		Self {
			shape,
			collider_type: ColliderType::Terrain,
			blocker_types: HashSet::from([Blocker::Physical]),
			center_offset: Vec3::ZERO,
			rotation: Quat::IDENTITY,
		}
	}

	pub fn with_center_offset(mut self, center_offset: Vec3) -> Self {
		self.center_offset = center_offset;
		self
	}

	pub fn with_rotation(mut self, rotation: Quat) -> Self {
		self.rotation = rotation;
		self
	}

	pub fn with_collider_type(mut self, collider_type: ColliderType) -> Self {
		self.collider_type = collider_type;
		self
	}

	pub fn with_blocker_types<TBlocks>(mut self, blocks: TBlocks) -> Self
	where
		TBlocks: IntoIterator<Item = Blocker>,
	{
		self.blocker_types = HashSet::from_iter(blocks);
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
///
/// All fields that apply to the same geometric dimension are to
/// be interpreted additively in order to prevent illogical value combinations.
/// For instance a capsule collider's full height is composed of `2 * half_y + 2 * radius`.
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Shape {
	Sphere {
		radius: Units,
	},
	Cuboid {
		half_x: Units,
		half_y: Units,
		half_z: Units,
	},
	Capsule {
		half_y: Units,
		radius: Units,
	},
	Cylinder {
		half_y: Units,
		radius: Units,
	},
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum ColliderType {
	Agent,
	Terrain,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum Blocker {
	Physical,
	Force,
	Character,
}

impl Blocker {
	pub fn all<TBlockers>() -> TBlockers
	where
		TBlockers: FromIterator<Blocker>,
	{
		Blocker::iterator().collect()
	}

	pub fn none<TBlockers>() -> TBlockers
	where
		TBlockers: FromIterator<Blocker>,
	{
		std::iter::empty().collect()
	}
}

impl IterFinite for Blocker {
	fn iterator() -> Iter<Self> {
		Iter(Some(Blocker::Physical))
	}

	fn next(current: &Iter<Self>) -> Option<Self> {
		match current.0? {
			Blocker::Physical => Some(Blocker::Force),
			Blocker::Force => Some(Blocker::Character),
			Blocker::Character => None,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn iterate() {
		assert_eq!(
			vec![Blocker::Physical, Blocker::Force, Blocker::Character],
			Blocker::iterator().take(100).collect::<Vec<_>>()
		);
	}
}
