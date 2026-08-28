use crate::math::f32_not_nan::{F32Invalid, F32NotNanBase};
use serde::Serialize;
use std::marker::PhantomData;

#[derive(Debug, PartialEq, Clone, Copy, Serialize)]
pub struct Positive;

impl Positive {
	pub(super) const fn parse_f32(
		F32NotNanBase(v, ..): F32NotNanBase,
	) -> Result<F32NotNanBase, F32Invalid> {
		if !v.is_sign_positive() {
			return Err(F32Invalid::Negative(v));
		}

		Ok(F32NotNanBase(v, PhantomData))
	}
}

#[macro_export]
macro_rules! f32_positive {
	($value:literal) => {{
		const F32: $crate::prelude::F32Positive =
			match $crate::prelude::F32Positive::try_from_f32($value) {
				Ok(v) => v,
				Err(_) => panic!("The f32 value is not positive"),
			};
		F32
	}};
}

pub use f32_positive;

#[macro_export]
macro_rules! f32_finite_positive {
	($value:literal) => {{
		const F32: $crate::prelude::F32FinitePositive =
			match $crate::prelude::F32FinitePositive::try_from_f32($value) {
				Ok(v) => v,
				Err(_) => panic!("The f32 value is not finite positive"),
			};
		F32
	}};
}

pub use f32_finite_positive;
