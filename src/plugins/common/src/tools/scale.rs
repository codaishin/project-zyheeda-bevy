use bevy::prelude::*;
use macros::serde_model;
use zyheeda_core::prelude::*;

#[macro_export]
macro_rules! scale {
	($($c:literal),+ $(,)?) => {{
		type T = zyheeda_core::prelude::F32FiniteStrictlyPositive;

		$crate::tools::scale::Scale([
			$(zyheeda_core::prelude::new_f32!(T($c)),)+
		])
	}};
}

pub use scale;

#[serde_model]
#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
pub struct Scale<const N: usize>(
	#[serde(with = "array_as_vec")] pub [F32FiniteStrictlyPositive; N],
);

impl<const N: usize> Scale<N> {
	pub const ONE: Scale<N> = Scale([new_f32!(F32FiniteStrictlyPositive(1.)); N]);
}

impl<const N: usize> Default for Scale<N> {
	fn default() -> Self {
		Self::ONE
	}
}

impl<const N: usize> TryFrom<[f32; N]> for Scale<N> {
	type Error = F32ParseError;

	fn try_from(array: [f32; N]) -> Result<Self, Self::Error> {
		let mut vec = [new_f32!(F32FiniteStrictlyPositive(1.)); N];

		for i in 0..N {
			vec[i] = F32FiniteStrictlyPositive::try_from_f32(array[i])?;
		}

		Ok(Self(vec))
	}
}

impl TryFrom<Vec2> for Scale<2> {
	type Error = F32ParseError;

	fn try_from(value: Vec2) -> Result<Self, Self::Error> {
		Self::try_from(value.to_array())
	}
}

impl From<Scale<2>> for Vec2 {
	fn from(Scale(scale): Scale<2>) -> Self {
		Vec2::from_array(scale.map(|s| *s))
	}
}

impl TryFrom<Vec3> for Scale<3> {
	type Error = F32ParseError;

	fn try_from(value: Vec3) -> Result<Self, Self::Error> {
		Self::try_from(value.to_array())
	}
}

impl From<Scale<3>> for Vec3 {
	fn from(Scale(scale): Scale<3>) -> Self {
		Vec3::from_array(scale.map(|s| *s))
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use test_case::test_case;

	#[test]
	fn scale_ok() {
		assert_eq!(
			Ok(Scale([
				new_f32!(F32FiniteStrictlyPositive(1.)),
				new_f32!(F32FiniteStrictlyPositive(2.)),
				new_f32!(F32FiniteStrictlyPositive(3.)),
			])),
			Scale::try_from([1., 2., 3.])
		);
	}

	#[test]
	fn scale_macro() {
		assert_eq!(
			Scale([
				new_f32!(F32FiniteStrictlyPositive(1.)),
				new_f32!(F32FiniteStrictlyPositive(2.)),
				new_f32!(F32FiniteStrictlyPositive(3.)),
			]),
			scale!(1., 2., 3.)
		);
	}

	#[test_case(f32::NAN, IsNaN([f32::NAN]); "nan")]
	#[test_case(f32::INFINITY, F32Invalid::Infinite(f32::INFINITY); "infinity")]
	#[test_case(f32::NEG_INFINITY, F32Invalid::Infinite(f32::NEG_INFINITY); "neg infinity")]
	#[test_case(-1., F32Invalid::Negative(-1.); "neg")]
	fn parse_error(value: f32, error: impl Into<F32ParseError>) {
		assert_eq!(Err(error.into()), Scale::try_from([value]));
	}
}
