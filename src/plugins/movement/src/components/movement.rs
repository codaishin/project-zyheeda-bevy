use bevy::prelude::*;
use common::traits::handles_movement::MovementTarget;
use macros::SavableComponent;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[savable_component(id = "movement")]
pub(crate) enum Movement {
	None,
	Direction(Dir3),
	Target(Vec3),
	Path(VecDeque<Vec3>),
}

#[cfg(test)]
impl Movement {
	pub(crate) fn path(path: impl Into<VecDeque<Vec3>>) -> Self {
		Self::Path(path.into())
	}
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
