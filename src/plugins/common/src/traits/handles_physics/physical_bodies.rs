use crate::{
	tools::Units,
	traits::iteration::{Iter, IterFinite},
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct BodyConfig {
	pub shape: Shape,
	pub physics_type: PhysicsType,
	pub sub_frames: Vec<InteractiveFrame>,
}

impl BodyConfig {
	pub fn from_shape(shape: Shape) -> Self {
		Self {
			shape,
			physics_type: PhysicsType::Terrain(HashSet::from([Blocker::Physical])),
			sub_frames: vec![],
		}
	}

	pub fn with_physics_type(mut self, physics_type: PhysicsType) -> Self {
		self.physics_type = physics_type;
		self
	}

	pub fn with_sub_frames(mut self, body_parts: impl Into<Vec<InteractiveFrame>>) -> Self {
		self.sub_frames = body_parts.into();
		self
	}
}

impl From<Shape> for BodyConfig {
	fn from(shape: Shape) -> Self {
		Self::from_shape(shape)
	}
}

#[derive(Debug, PartialEq, Default, Clone, Copy, Serialize, Deserialize)]
pub struct InteractiveFrame {
	pub forward_offset: Units,
	pub shape: ShapeParameters,
}

impl InteractiveFrame {
	pub const fn from_shape(shape: ShapeParameters) -> Self {
		Self {
			shape,
			forward_offset: Units::ZERO,
		}
	}

	pub const fn with_forward_offset(mut self, forward_offset: Units) -> Self {
		self.forward_offset = forward_offset;
		self
	}
}

impl From<ShapeParameters> for InteractiveFrame {
	fn from(shape: ShapeParameters) -> Self {
		Self::from_shape(shape)
	}
}

/// Shape definition. Used to describe physics colliders.
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Shape {
	/// Use the given parameters for a static collider
	Parameters(ShapeParameters),
	/// Use the [`Entity`]'s [`Mesh3d`] for a static collider
	StaticGltfMesh3d,
}

impl From<ShapeParameters> for Shape {
	fn from(shape: ShapeParameters) -> Self {
		Self::Parameters(shape)
	}
}

/// Shape parameters for a static collider.
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
pub enum ShapeParameters {
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

impl Default for ShapeParameters {
	fn default() -> Self {
		Self::Sphere {
			radius: Units::from(0.5),
		}
	}
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum PhysicsType {
	Agent(HashSet<Blocker>),
	Terrain(HashSet<Blocker>),
	InteractiveFrame,
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

impl IntoIterator for Blocker {
	type Item = Self;
	type IntoIter = std::iter::Once<Self>;

	fn into_iter(self) -> Self::IntoIter {
		std::iter::once(self)
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
