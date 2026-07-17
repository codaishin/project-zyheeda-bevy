use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, fmt::Display, hash::Hash, ops::Deref};

#[derive(Debug, PartialEq, Clone, Copy, Default, Serialize)]
pub struct F32Finite(f32);

impl F32Finite {
	pub const ZERO: Self = Self(0.);

	pub const fn try_from_f32(value: f32) -> Result<Self, NotFinite> {
		if value.is_nan() {
			return Err(NotFinite::NaN);
		}

		if value == f32::INFINITY {
			return Err(NotFinite::Infinity);
		}

		if value == f32::NEG_INFINITY {
			return Err(NotFinite::NegInfinity);
		}

		Ok(Self(value))
	}
}

#[macro_export]
macro_rules! f32_finite {
	($value:literal) => {{
		const F32_FINITE: $crate::prelude::F32Finite =
			match $crate::prelude::F32Finite::try_from_f32($value) {
				Ok(v) => v,
				Err(err) => panic!("{}", err.display()),
			};
		F32_FINITE
	}};
}

pub use f32_finite;

impl TryFrom<f32> for F32Finite {
	type Error = NotFinite;

	fn try_from(value: f32) -> Result<Self, Self::Error> {
		Self::try_from_f32(value)
	}
}

impl<'de> Deserialize<'de> for F32Finite {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let value = f32::deserialize(deserializer)?;

		F32Finite::try_from(value).map_err(NotFinite::into_serde_error)
	}
}

impl Deref for F32Finite {
	type Target = f32;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Eq for F32Finite {}

impl Hash for F32Finite {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		let bits = match self.0 {
			0. => 0,
			v => v.to_bits(),
		};

		bits.hash(state);
	}
}

impl Ord for F32Finite {
	fn cmp(&self, other: &Self) -> Ordering {
		match (self.0, other.0) {
			(0., 0.) => Ordering::Equal,
			(a, b) => a.total_cmp(&b),
		}
	}
}

impl PartialOrd for F32Finite {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum NotFinite {
	NaN,
	Infinity,
	NegInfinity,
}

impl NotFinite {
	const NAN: &str = "The f32 value is not a number";
	const INF: &str = "The f32 value is infinity";
	const NEG_INF: &str = "The f32 value is negative infinity";

	fn into_serde_error<TError>(self) -> TError
	where
		TError: serde::de::Error,
	{
		TError::custom(self)
	}

	pub const fn display(&self) -> &'static str {
		match self {
			Self::NaN => Self::NAN,
			Self::Infinity => Self::INF,
			Self::NegInfinity => Self::NEG_INF,
		}
	}
}

impl Display for NotFinite {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.display())
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
		let v = F32Finite::try_from(11.);
		assert_eq!(Ok(F32Finite(11.)), v);
	}

	#[test_case(f32::NAN, NotFinite::NaN; "nan")]
	#[test_case(f32::INFINITY, NotFinite::Infinity; "infinity")]
	#[test_case(f32::NEG_INFINITY, NotFinite::NegInfinity; "negative infinity")]
	fn try_from_error(v: f32, error: NotFinite) {
		let v = F32Finite::try_from(v);

		assert_eq!(Err(error), v);
	}

	#[test]
	fn deserialize() -> Result<(), yaml_serde::Error> {
		let yaml = "42";

		let value = yaml_serde::from_str::<F32Finite>(yaml)?;

		assert_eq!(F32Finite(42.), value);
		Ok(())
	}

	#[test_case(".NAN", NotFinite::NaN; "nan")]
	#[test_case(".INF", NotFinite::Infinity; "infinity")]
	#[test_case("-.INF", NotFinite::NegInfinity; "negative infinity")]
	fn deserialize_error(yaml: &str, error: NotFinite) {
		let value = yaml_serde::from_str::<F32Finite>(yaml);

		assert_eq!(
			Err(error.display().to_owned()),
			value.map_err(|e| e.to_string())
		);
	}

	#[test]
	fn macro_ok() {
		const V: F32Finite = f32_finite!(11.);

		assert_eq!(F32Finite(11.), V);
	}

	fn hash(node: impl Hash) -> u64 {
		let mut hasher = DefaultHasher::new();
		node.hash(&mut hasher);
		hasher.finish()
	}

	#[test]
	fn hash_value() {
		assert_eq!(hash(11f32.to_bits()), hash(F32Finite(11.)));
	}

	#[test]
	fn hash_zero() {
		assert_eq!(hash(F32Finite(-0.)), hash(F32Finite(0.)));
	}

	#[test_case(F32Finite(11.), F32Finite(10.), Ordering::Greater; "11 greater 10")]
	#[test_case(F32Finite(10.), F32Finite(11.), Ordering::Less; "10 less 11")]
	#[test_case(F32Finite(11.), F32Finite(11.), Ordering::Equal; "11 equal 11")]
	#[test_case(F32Finite(-0.), F32Finite(0.), Ordering::Equal; "0 equal 0")]
	fn order(a: F32Finite, b: F32Finite, ordering: Ordering) {
		assert_eq!(ordering, a.cmp(&b))
	}
}
