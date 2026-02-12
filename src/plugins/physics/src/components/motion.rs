use bevy::prelude::*;
use common::{
	tools::Done,
	traits::{accessors::get::GetProperty, handles_physics::LinearMotion},
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[cfg(test)]
use testing::ApproxEqual;

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[component(immutable)]
pub enum Motion {
	Ongoing(LinearMotion),
	Done(LinearMotion),
}

impl From<LinearMotion> for Motion {
	fn from(linear_motion: LinearMotion) -> Self {
		Self::Ongoing(linear_motion)
	}
}

impl GetProperty<LinearMotion> for Motion {
	fn get_property(&self) -> LinearMotion {
		match self {
			Motion::Ongoing(linear_motion) => *linear_motion,
			Motion::Done(linear_motion) => *linear_motion,
		}
	}
}

impl GetProperty<Done> for Motion {
	fn get_property(&self) -> bool {
		matches!(self, Motion::Done(..))
	}
}

#[cfg(test)]
impl ApproxEqual<f32> for Motion {
	fn approx_equal(&self, other: &Self, tolerance: &f32) -> bool {
		match (self, other) {
			(Motion::Ongoing(a), Motion::Ongoing(b)) => approx_equal(a, b, tolerance),
			(Motion::Done(a), Motion::Done(b)) => approx_equal(a, b, tolerance),
			_ => false,
		}
	}
}

/// Matches all [LinearMotion] variations pairs without silently falling through when new variations
/// are added
#[cfg(test)]
macro_rules! linear_motion_pairs {
	() => {
		(
			LinearMotion::Direction { .. } | LinearMotion::ToTarget { .. } | LinearMotion::Stop,
			_,
		)
	};
}

#[cfg(test)]
fn approx_equal(a: &LinearMotion, b: &LinearMotion, tolerance: &f32) -> bool {
	match (a, b) {
		(
			LinearMotion::Direction {
				speed: speed_a,
				direction: dir_a,
			},
			LinearMotion::Direction {
				speed: speed_b,
				direction: dir_b,
			},
		) => speed_a.approx_equal(speed_b, tolerance) && dir_a.approx_equal(dir_b, tolerance),

		(
			LinearMotion::ToTarget {
				speed: speed_a,
				target: target_a,
			},
			LinearMotion::ToTarget {
				speed: speed_b,
				target: target_b,
			},
		) => speed_a.approx_equal(speed_b, tolerance) && target_a.approx_equal(target_b, tolerance),
		(LinearMotion::Stop, LinearMotion::Stop) => true,
		linear_motion_pairs!() => false,
	}
}
