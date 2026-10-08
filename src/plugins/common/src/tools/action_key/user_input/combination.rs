use bevy::prelude::*;
use macros::serde_model;
use serde::Deserialize;
use std::collections::HashSet;
use zyheeda_core::prelude::*;

#[serde_model(no_default_deserialize)]
#[derive(Debug, PartialEq)]
pub struct UserInputCombination<const N: usize>(#[serde(with = "array_as_vec")] [ComboInput; N]);

impl UserInputCombination<2> {
	pub const ROTATE_DEBUG_NAV_MESH: Self = Self([
		ComboInput::KeyCode(KeyCode::ShiftLeft),
		ComboInput::KeyCode(KeyCode::F10),
	]);

	pub const ROTATE_DEBUG_COLLIDERS: Self = Self([
		ComboInput::KeyCode(KeyCode::ShiftLeft),
		ComboInput::KeyCode(KeyCode::F11),
	]);
}

impl<const N: usize> TryFrom<[ComboInput; N]> for UserInputCombination<N> {
	type Error = InputRepeated;

	fn try_from(input: [ComboInput; N]) -> Result<Self, Self::Error> {
		let mut seen = HashSet::from([]);

		for input in input.iter().copied() {
			if !seen.insert(input) {
				return Err(InputRepeated(input));
			}
		}

		Ok(Self(input))
	}
}

impl<'de, const N: usize> Deserialize<'de> for UserInputCombination<N> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let combos = array_as_vec::deserialize(deserializer)?;

		Self::try_from(combos)
			.map_err(|_| serde::de::Error::custom("Encountered duplicate combo input keys"))
	}
}

#[serde_model]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ComboInput {
	KeyCode(KeyCode),
	MouseButton(MouseButton),
}

#[derive(Debug, PartialEq)]
pub struct InputRepeated(ComboInput);

#[cfg(test)]
mod tests {
	use super::*;
	use test_case::test_case;

	#[test]
	fn map_values() -> Result<(), InputRepeated> {
		let UserInputCombination(c) = UserInputCombination::try_from([
			ComboInput::KeyCode(KeyCode::KeyA),
			ComboInput::MouseButton(MouseButton::Right),
		])?;

		assert_eq!(
			[
				ComboInput::KeyCode(KeyCode::KeyA),
				ComboInput::MouseButton(MouseButton::Right)
			],
			c
		);
		Ok(())
	}

	#[test]
	fn reject_repeating() {
		let c = UserInputCombination::try_from([
			ComboInput::KeyCode(KeyCode::KeyA),
			ComboInput::KeyCode(KeyCode::KeyA),
		]);

		assert_eq!(Err(InputRepeated(ComboInput::KeyCode(KeyCode::KeyA))), c);
	}

	#[test_case(UserInputCombination::ROTATE_DEBUG_NAV_MESH; "rotate debug nav mesh")]
	#[test_case(UserInputCombination::ROTATE_DEBUG_COLLIDERS; "rotate debug colliders")]
	fn const_not_repeating<const N: usize>(UserInputCombination(inputs): UserInputCombination<N>) {
		assert!(UserInputCombination::try_from(inputs).is_ok());
	}
}
