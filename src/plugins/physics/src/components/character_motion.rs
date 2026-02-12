use bevy::prelude::*;
use common::{
	tools::Done,
	traits::{accessors::get::GetProperty, handles_physics::CharacterMotion},
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[cfg(test)]
use testing::ApproxEqual;

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[component(immutable)]
pub enum ApplyCharacterMotion {
	Ongoing(CharacterMotion),
	Done(CharacterMotion),
}

impl From<CharacterMotion> for ApplyCharacterMotion {
	fn from(linear_motion: CharacterMotion) -> Self {
		Self::Ongoing(linear_motion)
	}
}

impl GetProperty<CharacterMotion> for ApplyCharacterMotion {
	fn get_property(&self) -> CharacterMotion {
		match self {
			ApplyCharacterMotion::Ongoing(linear_motion) => *linear_motion,
			ApplyCharacterMotion::Done(linear_motion) => *linear_motion,
		}
	}
}

impl GetProperty<Done> for ApplyCharacterMotion {
	fn get_property(&self) -> bool {
		matches!(self, ApplyCharacterMotion::Done(..))
	}
}

#[cfg(test)]
impl ApproxEqual<f32> for ApplyCharacterMotion {
	fn approx_equal(&self, other: &Self, tolerance: &f32) -> bool {
		match (self, other) {
			(ApplyCharacterMotion::Ongoing(a), ApplyCharacterMotion::Ongoing(b)) => {
				approx_equal(a, b, tolerance)
			}
			(ApplyCharacterMotion::Done(a), ApplyCharacterMotion::Done(b)) => {
				approx_equal(a, b, tolerance)
			}
			_ => false,
		}
	}
}

/// Matches all [CharacterMotion] pair permutations without silently falling through when new
/// variations are added
#[cfg(test)]
macro_rules! remaining_character_motion_permutations {
	() => {
		(
			CharacterMotion::Direction { .. }
				| CharacterMotion::ToTarget { .. }
				| CharacterMotion::Stop,
			_,
		)
	};
}

#[cfg(test)]
fn approx_equal(a: &CharacterMotion, b: &CharacterMotion, tolerance: &f32) -> bool {
	match (a, b) {
		(
			CharacterMotion::Direction {
				speed: speed_a,
				direction: dir_a,
			},
			CharacterMotion::Direction {
				speed: speed_b,
				direction: dir_b,
			},
		) => speed_a.approx_equal(speed_b, tolerance) && dir_a.approx_equal(dir_b, tolerance),

		(
			CharacterMotion::ToTarget {
				speed: speed_a,
				target: target_a,
			},
			CharacterMotion::ToTarget {
				speed: speed_b,
				target: target_b,
			},
		) => speed_a.approx_equal(speed_b, tolerance) && target_a.approx_equal(target_b, tolerance),
		(CharacterMotion::Stop, CharacterMotion::Stop) => true,
		remaining_character_motion_permutations!() => false,
	}
}
