use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, fmt::Display, hash::Hash, ops::Deref};

#[derive(Debug, PartialEq, Clone, Copy, Default, Serialize)]
pub struct F32NotNan(f32);

impl F32NotNan {
	pub const ZERO: Self = Self(0.);

	pub const fn try_from_f32(value: f32) -> Result<Self, IsNaN> {
		if value.is_nan() {
			return Err(IsNaN);
		}

		Ok(Self(value))
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

impl TryFrom<f32> for F32NotNan {
	type Error = IsNaN;

	fn try_from(value: f32) -> Result<Self, Self::Error> {
		Self::try_from_f32(value)
	}
}

impl<'de> Deserialize<'de> for F32NotNan {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let value = f32::deserialize(deserializer)?;

		F32NotNan::try_from(value).map_err(IsNaN::into_serde_error)
	}
}

impl Deref for F32NotNan {
	type Target = f32;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Eq for F32NotNan {}

impl Hash for F32NotNan {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		let bits = match self.0 {
			0. => 0,
			v => v.to_bits(),
		};

		bits.hash(state);
	}
}

impl Ord for F32NotNan {
	fn cmp(&self, other: &Self) -> Ordering {
		match (self.0, other.0) {
			(0., 0.) => Ordering::Equal,
			(a, b) => a.total_cmp(&b),
		}
	}
}

impl PartialOrd for F32NotNan {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct IsNaN;

impl IsNaN {
	fn into_serde_error<TError>(self) -> TError
	where
		TError: serde::de::Error,
	{
		TError::custom(self)
	}
}

impl Display for IsNaN {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "The f32 value is not a number")
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::{
		cmp::Ordering,
		hash::{DefaultHasher, Hasher},
	};
	use test_case::test_case;

	#[test]
	fn try_from_ok() {
		let v = F32NotNan::try_from(11.);

		assert_eq!(Ok(F32NotNan(11.)), v);
	}

	#[test]
	fn try_from_error() {
		let v = F32NotNan::try_from(f32::NAN);

		assert_eq!(Err(IsNaN), v);
	}

	#[test]
	fn deserialize() -> Result<(), yaml_serde::Error> {
		let yaml = "42";

		let value = yaml_serde::from_str::<F32NotNan>(yaml)?;

		assert_eq!(F32NotNan(42.), value);
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

		assert_eq!(F32NotNan(11.), V);
	}

	fn hash(node: impl Hash) -> u64 {
		let mut hasher = DefaultHasher::new();
		node.hash(&mut hasher);
		hasher.finish()
	}

	#[test]
	fn hash_value() {
		assert_eq!(hash(11f32.to_bits()), hash(F32NotNan(11.)));
	}

	#[test]
	fn hash_zero() {
		assert_eq!(hash(F32NotNan(-0.)), hash(F32NotNan(0.)));
	}

	#[test_case(F32NotNan(11.), F32NotNan(10.), Ordering::Greater; "11 greater 10")]
	#[test_case(F32NotNan(10.), F32NotNan(11.), Ordering::Less; "10 less 11")]
	#[test_case(F32NotNan(11.), F32NotNan(11.), Ordering::Equal; "11 equal 11")]
	#[test_case(F32NotNan(-0.), F32NotNan(0.), Ordering::Equal; "0 equal 0")]
	fn order(a: F32NotNan, b: F32NotNan, ordering: Ordering) {
		assert_eq!(ordering, a.cmp(&b))
	}
}
