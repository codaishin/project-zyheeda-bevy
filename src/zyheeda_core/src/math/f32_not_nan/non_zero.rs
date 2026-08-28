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
