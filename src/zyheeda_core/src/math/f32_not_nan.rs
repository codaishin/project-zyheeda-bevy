pub mod finite;
pub mod positive;

use crate::math::f32_not_nan::{finite::Finite, positive::Positive};
use macros::serde_model;
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, fmt::Display, hash::Hash, marker::PhantomData, ops::Deref};

#[serde_model(no_default_deserialize)]
#[derive(Debug)]
pub struct F32NotNanBase<TConstraint = ()>(f32, PhantomData<TConstraint>)
where
	TConstraint: Serialize;

impl F32NotNanBase {
	pub const ZERO: Self = Self(0., PhantomData);

	pub const fn try_from_f32(value: f32) -> Result<Self, IsNaN<1>> {
		if value.is_nan() {
			return Err(IsNaN([value]));
		}

		Ok(Self(value, PhantomData))
	}
}

#[macro_export]
macro_rules! f32_not_nan {
	($value:literal) => {{
		const F32_NOT_NAN: $crate::prelude::F32NotNan =
			match $crate::prelude::F32NotNan::try_from_f32($value) {
				Ok(v) => v,
				Err(_) => panic!("The f32 value is not a number"),
			};
		F32_NOT_NAN
	}};
}

pub use f32_not_nan;

impl Default for F32NotNanBase {
	fn default() -> Self {
		Self::ZERO
	}
}

impl TryFrom<f32> for F32NotNanBase {
	type Error = IsNaN<1>;

	fn try_from(value: f32) -> Result<Self, Self::Error> {
		Self::try_from_f32(value)
	}
}

impl<'de> Deserialize<'de> for F32NotNanBase {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let value = f32::deserialize(deserializer)?;

		F32NotNan::try_from(value).map_err(IsNaN::into_serde_error)
	}
}

impl<TConstraint> Deref for F32NotNanBase<TConstraint>
where
	TConstraint: Serialize,
{
	type Target = f32;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<TConstraint> Clone for F32NotNanBase<TConstraint>
where
	TConstraint: Serialize,
{
	fn clone(&self) -> Self {
		*self
	}
}

impl<TConstraint> Copy for F32NotNanBase<TConstraint> where TConstraint: Serialize {}

impl<TConstraint> PartialEq for F32NotNanBase<TConstraint>
where
	TConstraint: Serialize,
{
	fn eq(&self, other: &Self) -> bool {
		self.0 == other.0
	}
}

impl<TConstraint> Eq for F32NotNanBase<TConstraint> where TConstraint: Serialize {}

impl<TConstraint> Hash for F32NotNanBase<TConstraint>
where
	TConstraint: Serialize,
{
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		let bits = match self.0 {
			0. => 0,
			v => v.to_bits(),
		};

		bits.hash(state);
	}
}

impl<TConstraint> Ord for F32NotNanBase<TConstraint>
where
	TConstraint: Serialize,
{
	fn cmp(&self, other: &Self) -> Ordering {
		match (self.0, other.0) {
			(0., 0.) => Ordering::Equal,
			(a, b) => a.total_cmp(&b),
		}
	}
}

impl<TConstraint> PartialOrd for F32NotNanBase<TConstraint>
where
	TConstraint: Serialize,
{
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

#[derive(Debug, Clone)]
pub struct IsNaN<const N: usize>(pub [f32; N]);

impl<const N: usize> PartialEq for IsNaN<N> {
	fn eq(&self, other: &Self) -> bool {
		for (s, o) in self.0.iter().zip(other.0.iter()) {
			if s.is_nan() && o.is_nan() {
				continue;
			}

			if s != o {
				return false;
			}
		}

		true
	}
}

impl From<IsNaN<1>> for F32ParseError {
	fn from(nan: IsNaN<1>) -> Self {
		Self::IsNaN(nan)
	}
}

impl<const N: usize> IsNaN<N> {
	fn into_serde_error<TError>(self) -> TError
	where
		TError: serde::de::Error,
	{
		TError::custom(self)
	}
}

impl<const N: usize> Display for IsNaN<N> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let IsNaN(values) = self;
		write!(f, "{values:?}: contains elements that are not a number")
	}
}

#[derive(Debug, PartialEq)]
pub enum F32ParseError {
	IsNaN(IsNaN<1>),
	Invalid(F32Invalid),
}

impl F32ParseError {
	fn into_serde_error<TError>(self) -> TError
	where
		TError: serde::de::Error,
	{
		TError::custom(self)
	}
}

impl Display for F32ParseError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			F32ParseError::IsNaN(err) => err.fmt(f),
			F32ParseError::Invalid(F32Invalid::Infinite(v)) => write!(f, "{v}: is infinite"),
			F32ParseError::Invalid(F32Invalid::Negative(v)) => write!(f, "{v}: is negative"),
		}
	}
}

#[derive(Debug, PartialEq)]
pub enum F32Invalid {
	Infinite(f32),
	Negative(f32),
}

impl From<F32Invalid> for F32ParseError {
	fn from(invalid: F32Invalid) -> Self {
		Self::Invalid(invalid)
	}
}

macro_rules! impl_constraints {
	($t_fst:ty $(, $t_rest:ty)*) => {
		#[allow(unused_parens)]
		impl F32NotNanBase<($t_fst $(, $t_rest)*)>
		where
			$t_fst: Serialize,
			$($t_rest: Serialize),*
		{
			pub const fn try_from_f32(value: f32) -> Result<Self, F32ParseError>

			 {
				let mut value = match F32NotNan::try_from_f32(value) {
					Ok(value) => value,
					Err(err) => return Err(F32ParseError::IsNaN(err)),
				};

				value = match <$t_fst>::parse_f32(value) {
					Ok(value) => value,
					Err(err) => return Err(F32ParseError::Invalid(err)),
				};
				$(
					value = match <$t_rest>::parse_f32(value) {
						Ok(value) => value,
						Err(err) => return Err(F32ParseError::Invalid(err)),
					};
				)*


				Ok(Self(value.0, PhantomData))
			}
		}

		#[allow(unused_parens)]
		impl TryFrom<f32> for F32NotNanBase<($t_fst $(, $t_rest)*)>
		where
			$t_fst: Serialize,
			$($t_rest: Serialize),*
		{
			type Error = F32ParseError;

			fn try_from(value: f32) -> Result<Self, Self::Error> {
				Self::try_from_f32(value)
			}
		}

		#[allow(unused_parens)]
		impl<'de> Deserialize<'de> for F32NotNanBase<($t_fst $(, $t_rest)*)>
		where
			$t_fst: Serialize,
			$($t_rest: Serialize),*
		{
			fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
			where
				D: serde::Deserializer<'de>,
			{
				let value = f32::deserialize(deserializer)?;

				F32NotNanBase::<($t_fst $(, $t_rest)*)>
					::try_from_f32(value)
					.map_err(F32ParseError::into_serde_error)
			}
		}
	};
}

pub type F32NotNan = F32NotNanBase;
pub type F32Finite = F32NotNanBase<Finite>;
pub type F32Positive = F32NotNanBase<Positive>;
pub type F32FinitePositive = F32NotNanBase<(Finite, Positive)>;

impl_constraints!(Finite);
impl_constraints!(Positive);
impl_constraints!(Finite, Positive);

#[cfg(test)]
mod tests {
	use super::*;
	use std::{
		cmp::Ordering,
		hash::{DefaultHasher, Hasher},
	};
	use test_case::test_case;

	#[cfg(test)]
	impl<TConstraint> F32NotNanBase<TConstraint>
	where
		TConstraint: Serialize,
	{
		fn raw(raw: f32) -> Self {
			Self(raw, PhantomData)
		}
	}

	#[test]
	fn try_from_ok() {
		let v = F32NotNan::try_from(11.);

		assert_eq!(Ok(F32NotNan::raw(11.)), v);
	}

	#[test]
	fn try_from_error() {
		let v = F32NotNan::try_from(f32::NAN);

		assert_eq!(Err(IsNaN([f32::NAN])), v);
	}

	#[test_case(f32::INFINITY, Err(F32ParseError::Invalid(F32Invalid::Infinite(f32::INFINITY))); "positive")]
	#[test_case(f32::NEG_INFINITY, Err(F32ParseError::Invalid(F32Invalid::Infinite(f32::NEG_INFINITY))); "negative")]
	#[test_case(11., Ok(F32Finite::raw(11.)); "ok")]
	fn try_from_finite(v: f32, result: Result<F32Finite, F32ParseError>) {
		let v = F32Finite::try_from_f32(v);

		assert_eq!(result, v);
	}

	#[test_case(-1., Err(F32ParseError::Invalid(F32Invalid::Negative(-1.))); "err")]
	#[test_case(11., Ok(F32Positive::raw(11.)); "ok")]
	fn try_from_positive(v: f32, result: Result<F32Positive, F32ParseError>) {
		let v = F32Positive::try_from_f32(v);

		assert_eq!(result, v);
	}

	#[test_case(f32::INFINITY, Err(F32ParseError::Invalid(F32Invalid::Infinite(f32::INFINITY))); "positive inf")]
	#[test_case(f32::NEG_INFINITY, Err(F32ParseError::Invalid(F32Invalid::Infinite(f32::NEG_INFINITY))); "negative inf")]
	#[test_case(-1., Err(F32ParseError::Invalid(F32Invalid::Negative(-1.))); "negative")]
	#[test_case(11., Ok(F32FinitePositive::raw(11.)); "ok")]
	fn try_from_finite_positive(v: f32, result: Result<F32FinitePositive, F32ParseError>) {
		let v = F32FinitePositive::try_from_f32(v);

		assert_eq!(result, v);
	}

	#[test]
	fn deserialize() -> Result<(), yaml_serde::Error> {
		let yaml = "42";

		let value = yaml_serde::from_str::<F32NotNan>(yaml)?;

		assert_eq!(F32NotNan::raw(42.), value);
		Ok(())
	}

	#[test]
	fn deserialize_nan() {
		let yaml = ".NAN";

		let value = yaml_serde::from_str::<F32NotNan>(yaml);

		assert!(value.is_err());
	}

	#[test]
	fn macro_ok() {
		const V: F32NotNan = f32_not_nan!(11.);

		assert_eq!(F32NotNan::raw(11.), V);
	}

	fn hash(node: impl Hash) -> u64 {
		let mut hasher = DefaultHasher::new();
		node.hash(&mut hasher);
		hasher.finish()
	}

	#[test]
	fn hash_value() {
		assert_eq!(hash(11f32.to_bits()), hash(F32NotNan::raw(11.)));
	}

	#[test]
	fn hash_zero() {
		assert_eq!(hash(F32NotNan::raw(-0.)), hash(F32NotNan::raw(0.)));
	}

	#[test_case(F32NotNan::raw(11.), F32NotNan::raw(10.), Ordering::Greater; "11 greater 10")]
	#[test_case(F32NotNan::raw(10.), F32NotNan::raw(11.), Ordering::Less; "10 less 11")]
	#[test_case(F32NotNan::raw(11.), F32NotNan::raw(11.), Ordering::Equal; "11 equal 11")]
	#[test_case(F32NotNan::raw(-0.), F32NotNan::raw(0.), Ordering::Equal; "0 equal 0")]
	fn order(a: F32NotNan, b: F32NotNan, ordering: Ordering) {
		assert_eq!(ordering, a.cmp(&b))
	}
}
