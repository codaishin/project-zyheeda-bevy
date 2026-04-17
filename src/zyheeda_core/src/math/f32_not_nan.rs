use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, PartialEq, Clone, Copy, Serialize)]
pub struct F32NotNan(f32);

impl TryFrom<f32> for F32NotNan {
	type Error = IsNaN;

	fn try_from(value: f32) -> Result<Self, Self::Error> {
		if value.is_nan() {
			return Err(IsNaN);
		}

		Ok(Self(value))
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
}
