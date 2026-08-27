use crate::math::f32_not_nan::{F32Finite, F32Invalid, F32NotNanBase};
use serde::Serialize;
use std::marker::PhantomData;

#[derive(Debug, PartialEq, Clone, Copy, Serialize)]
pub struct Finite;

impl Finite {
	pub(super) const fn parse_f32(
		F32NotNanBase(v, ..): F32NotNanBase,
	) -> Result<F32NotNanBase, F32Invalid> {
		if v.is_infinite() {
			return Err(F32Invalid::Infinite(v));
		}

		Ok(F32NotNanBase(v, PhantomData))
	}
}

impl F32Finite {
	pub const ZERO: Self = Self(0., PhantomData);
}

impl Default for F32Finite {
	fn default() -> Self {
		Self(0., PhantomData)
	}
}

#[macro_export]
macro_rules! f32_finite {
	($value:literal) => {{
		const F32: $crate::prelude::F32Finite =
			match $crate::prelude::F32Finite::try_from_f32($value) {
				Ok(v) => v,
				Err(_) => panic!("The f32 value is not finite"),
			};
		F32
	}};
}

pub use f32_finite;
