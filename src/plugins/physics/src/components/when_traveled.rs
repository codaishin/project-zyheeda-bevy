use bevy::prelude::*;
use common::tools::Units;

#[derive(Debug, PartialEq)]
pub(crate) struct WhenTraveled {
	distance: Units,
}

impl WhenTraveled {
	pub(crate) fn distance(distance: Units) -> Self {
		Self { distance }
	}

	pub(crate) fn destroy(self) -> DestroyAfterDistanceTraveled {
		DestroyAfterDistanceTraveled {
			remaining_distance: self.distance,
		}
	}
}

#[derive(Component, Debug, PartialEq)]
pub(crate) struct DestroyAfterDistanceTraveled {
	pub(crate) remaining_distance: Units,
}
