use crate::{
	components::Player,
	traits::{clamp_zero_positive::ClampZeroPositive, load_asset::Path},
};
use macros::ClampZeroPositive;

///Serves as a struct to implement static traits on
pub struct Tools;

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug, Default, ClampZeroPositive)]
pub struct UnitsPerSecond(f32);

#[derive(Debug, PartialEq, Clone, ClampZeroPositive)]
pub struct Units(f32);

#[derive(Debug, PartialEq, Clone, ClampZeroPositive)]
pub struct Intensity(f32);

#[derive(Debug, PartialEq, Clone, ClampZeroPositive)]
pub struct IntensityChangePerSecond(f32);

pub fn player_animation_path(animation_name: &str) -> Path {
	Path::from(Player::MODEL_PATH.to_owned() + "#" + animation_name)
}

#[cfg(test)]
mod test_clamp_zero_positive_on_units_per_second {
	use super::*;

	#[test]
	fn set_value() {
		assert_eq!(42., UnitsPerSecond::new(42.).value());
	}

	#[test]
	fn min_zero() {
		assert_eq!(0., UnitsPerSecond::new(-42.).value());
	}
}
