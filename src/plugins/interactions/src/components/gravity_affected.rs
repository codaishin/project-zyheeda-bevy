use bevy::prelude::{Component, Entity};
use common::tools::UnitsPerSecond;
use std::{ops::RangeBounds, vec::Drain};

#[derive(Component, Debug, PartialEq, Clone, Default)]
pub struct GravityAffected {
	pulls: Vec<GravityPull>,
}

impl GravityAffected {
	#[cfg(test)]
	pub(crate) fn new<const N: usize>(pulls: [GravityPull; N]) -> Self {
		Self {
			pulls: Vec::from(pulls),
		}
	}

	pub(crate) fn is_not_pulled(&self) -> bool {
		self.pulls.is_empty()
	}

	pub(crate) fn push(&mut self, pull: GravityPull) {
		self.pulls.push(pull);
	}

	pub(crate) fn drain_pulls<TRange>(&mut self, range: TRange) -> Drain<GravityPull>
	where
		TRange: RangeBounds<usize>,
	{
		self.pulls.drain(range)
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct GravityPull {
	pub(crate) strength: UnitsPerSecond,
	pub(crate) towards: Entity,
}
