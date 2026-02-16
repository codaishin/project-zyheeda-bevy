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
#[savable_component(id = "apply character motion")]
#[require(IsInMotion)]
pub struct ApplyCharacterMotion {
	pub(crate) motion: CharacterMotion,
	pub(crate) is_done: bool,
}

impl From<CharacterMotion> for ApplyCharacterMotion {
	fn from(motion: CharacterMotion) -> Self {
		Self {
			motion,
			is_done: false,
		}
	}
}

impl GetProperty<CharacterMotion> for ApplyCharacterMotion {
	fn get_property(&self) -> CharacterMotion {
		self.motion
	}
}

impl GetProperty<Done> for ApplyCharacterMotion {
	fn get_property(&self) -> bool {
		self.is_done
	}
}

#[cfg(test)]
impl ApproxEqual<f32> for ApplyCharacterMotion {
	fn approx_equal(&self, other: &Self, tolerance: &f32) -> bool {
		approx_equal(&self.motion, &other.motion, tolerance) && self.is_done == other.is_done
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
