use bevy::prelude::Component;
use common::{
	components::persistent_entity::PersistentEntity,
	tools::UnitsPerSecond,
	traits::handles_saving::SavableComponent,
};
use serde::{Deserialize, Serialize};
use std::{ops::RangeBounds, vec::Drain};

#[derive(Component, Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct GravityAffected {
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
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

impl SavableComponent for GravityAffected {
	type TDto = Self;
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct GravityPull {
	pub(crate) strength: UnitsPerSecond,
	pub(crate) towards: PersistentEntity,
}
