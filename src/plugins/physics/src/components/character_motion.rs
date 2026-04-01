use bevy::prelude::*;
use common::traits::{accessors::get::View, handles_physics::CharacterMotion};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[cfg(test)]
use testing::ApproxEqual;

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[component(immutable)]
#[savable_component(id = "apply character motion")]
#[require(IsInMotion)]
pub struct ApplyCharacterMotion(pub(crate) CharacterMotion);

impl From<CharacterMotion> for ApplyCharacterMotion {
	fn from(motion: CharacterMotion) -> Self {
		Self(motion)
	}
}

impl View<CharacterMotion> for ApplyCharacterMotion {
	fn view(&self) -> CharacterMotion {
		self.0
	}
}

#[cfg(test)]
impl ApproxEqual<f32> for ApplyCharacterMotion {
	fn approx_equal(&self, other: &Self, tolerance: &f32) -> bool {
		approx_equal(&self.0, &other.0, tolerance)
	}
}

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct IsInMotion;

/// Matches all [CharacterMotion] pair permutations without silently falling through when new
/// variations are added
#[cfg(test)]
macro_rules! remaining_character_motion_permutations {
	() => {
		(
			CharacterMotion::Direction { .. }
				| CharacterMotion::ToTarget { .. }
				| CharacterMotion::Done,
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
		(CharacterMotion::Done, CharacterMotion::Done) => true,
		remaining_character_motion_permutations!() => false,
	}
}
