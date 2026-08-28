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
