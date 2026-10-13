use crate::traits::handles_localization::{Token, TokenItem};
use bevy::prelude::*;
use macros::serde_model;
use serde::Deserialize;
use std::{collections::HashSet, ops::Deref};
use zyheeda_core::prelude::*;

#[serde_model(no_default_deserialize)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
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

	pub fn iter(&self) -> std::slice::Iter<'_, ComboInput> {
		self.into_iter()
	}

	pub fn as_array(self) -> [ComboInput; 2] {
		self.0
	}
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

impl<const N: usize> From<UserInputCombination<N>> for Token {
	fn from(UserInputCombination(combination): UserInputCombination<N>) -> Self {
		let items = combination
			.into_iter()
			.map(|input| match input {
				ComboInput::KeyCode(key_code) => TokenItem::from(key_code),
				ComboInput::MouseButton(mouse_button) => TokenItem::from(mouse_button),
			})
			.intersperse_with_item(|| TokenItem::Raw(String::from(" + ")));

		Token::from_iter(items)
	}
}

impl<'a, const N: usize> IntoIterator for &'a UserInputCombination<N> {
	type Item = &'a ComboInput;
	type IntoIter = std::slice::Iter<'a, ComboInput>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.iter()
	}
}

impl<const N: usize> Deref for UserInputCombination<N> {
	type Target = [ComboInput];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

#[serde_model]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ComboInput {
	KeyCode(KeyCode),
	MouseButton(MouseButton),
}

impl From<KeyCode> for ComboInput {
	fn from(value: KeyCode) -> Self {
		Self::KeyCode(value)
	}
}

impl From<MouseButton> for ComboInput {
	fn from(value: MouseButton) -> Self {
		Self::MouseButton(value)
	}
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

	#[test]
	fn token() -> Result<(), InputRepeated> {
		let combo = UserInputCombination::try_from([
			ComboInput::KeyCode(KeyCode::KeyA),
			ComboInput::MouseButton(MouseButton::Right),
		])?;

		let token = Token::from(combo);

		assert_eq!(
			Token::from_iter([
				TokenItem::from(KeyCode::KeyA),
				TokenItem::Raw(String::from(" + ")),
				TokenItem::from(MouseButton::Right),
			]),
			token
		);
		Ok(())
	}
}
