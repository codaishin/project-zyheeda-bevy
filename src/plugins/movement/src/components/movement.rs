use bevy::prelude::*;
use common::traits::handles_movement::MovementTarget;
use macros::SavableComponent;
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, ops::Deref};

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[savable_component(id = "movement")]
pub(crate) enum Movement {
	None,
	Direction(Dir3),
	Target(Vec3),
	Path(MovementPath),
}

impl<T> From<T> for Movement
where
	T: Into<MovementTarget>,
{
	fn from(value: T) -> Self {
		match value.into() {
			MovementTarget::Dir(direction) => Self::Direction(direction),
			MovementTarget::Point(point) => Self::Target(point),
		}
	}
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub(crate) struct MovementPath {
	waypoints: VecDeque<Vec3>,
	is_new: bool,
}

impl MovementPath {
	pub(crate) fn pop_front(&mut self) -> Option<Vec3> {
		self.is_new = false;
		self.waypoints.pop_front()
	}

	pub(crate) fn is_new(&self) -> bool {
		self.is_new
	}

	#[cfg(test)]
	pub(crate) fn not_new(mut self) -> Self {
		self.is_new = false;
		self
	}
}

impl<const N: usize> From<[Vec3; N]> for MovementPath {
	fn from(waypoints: [Vec3; N]) -> Self {
		Self {
			waypoints: VecDeque::from(waypoints),
			is_new: true,
		}
	}
}

impl From<VecDeque<Vec3>> for MovementPath {
	fn from(waypoints: VecDeque<Vec3>) -> Self {
		Self {
			waypoints,
			is_new: true,
		}
	}
}

impl Deref for MovementPath {
	type Target = VecDeque<Vec3>;

	fn deref(&self) -> &Self::Target {
		&self.waypoints
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn path_pop_front() {
		let mut path = MovementPath::from([Vec3::new(1., 2., 3.), Vec3::new(2., 3., 4.)]);

		let wp = path.pop_front();

		assert_eq!(
			(
				Some(Vec3::new(1., 2., 3.)),
				MovementPath::from([Vec3::new(2., 3., 4.)]).not_new()
			),
			(wp, path)
		);
	}

	#[test]
	fn path_is_new() {
		let path = MovementPath::from([Vec3::new(1., 2., 3.), Vec3::new(2., 3., 4.)]);

		assert!(path.is_new())
	}

	#[test]
	fn path_not_new() {
		let mut path = MovementPath::from([Vec3::new(1., 2., 3.), Vec3::new(2., 3., 4.)]);

		path.pop_front();

		assert!(!path.is_new())
	}
}
