use crate::math::f32_not_nan::{F32Invalid, F32NotNanBase};
use serde::Serialize;
use std::marker::PhantomData;

#[derive(Debug, PartialEq, Clone, Copy, Serialize)]
pub struct NonZero;

impl NonZero {
	pub(super) const fn parse_f32(
		F32NotNanBase(v, ..): F32NotNanBase,
	) -> Result<F32NotNanBase, F32Invalid> {
		if v == 0. {
			return Err(F32Invalid::Zero(v));
		}

		Ok(F32NotNanBase(v, PhantomData))
	}
}

#[macro_export]
macro_rules! f32_finite_strictly_positive {
	($value:literal) => {{
		const F32: $crate::prelude::F32FiniteStrictlyPositive =
			match $crate::prelude::F32FiniteStrictlyPositive::try_from_f32($value) {
				Ok(v) => v,
				Err(_) => panic!("The f32 value is not finite and greater than zero"),
			};
		F32
	}};
}

pub use f32_finite_strictly_positive;

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::prelude::{F32NotNan, f32_not_nan};
	use test_case::test_case;

	#[test]
	fn non_zero_ok() {
		let v = f32_not_nan!(1.);

		assert_eq!(Ok(f32_not_nan!(1.)), NonZero::parse_f32(v));
	}

	#[test_case(0.0; "positive")]
	#[test_case(-0.0; "negative")]
	fn zero_error(v: f32) {
		let f32 = F32NotNan::try_from_f32(v).unwrap();

		assert_eq!(Err(F32Invalid::Zero(v)), NonZero::parse_f32(f32));
	}
}
