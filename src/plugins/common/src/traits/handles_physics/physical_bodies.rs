use crate::{
	tools::Units,
	traits::iteration::{Iter, IterFinite},
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub trait HandlesPhysicalBodies {
	type TBody: Component + From<Body>;
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Body {
	pub shape: Shape,
	pub physics_type: PhysicsType,
	pub blocker_types: HashSet<Blocker>,
}

impl Body {
	pub fn from_shape(shape: Shape) -> Self {
		Self {
			shape,
			physics_type: PhysicsType::Terrain,
			blocker_types: HashSet::from([Blocker::Physical]),
		}
	}

	pub fn with_physics_type(mut self, physics_type: PhysicsType) -> Self {
		self.physics_type = physics_type;
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

impl From<Shape> for Body {
	fn from(shape: Shape) -> Self {
		Self::from_shape(shape)
	}
}

/// Shape definition, usually used to describe physics colliders.
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
pub enum PhysicsType {
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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Blockers {
	All,
	AnyOf(HashSet<Blocker>),
}

impl From<Blockers> for HashSet<Blocker> {
	fn from(value: Blockers) -> Self {
		match value {
			Blockers::All => Blocker::all(),
			Blockers::AnyOf(blockers) => blockers,
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
