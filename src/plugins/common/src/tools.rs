use crate::traits::clamp_zero_positive::ClampZeroPositive;
use macros::ClampZeroPositive;

///Serves as a struct to implement static traits on
pub struct Tools;

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug, Default, ClampZeroPositive)]
pub struct UnitsPerSecond(f32);

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
