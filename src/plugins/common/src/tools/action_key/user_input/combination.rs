use crate::traits::{
	handles_input::RawInput,
	handles_localization::{Token, TokenItem},
};
use bevy::prelude::*;
use macros::serde_model;
use serde::Deserialize;
use std::{collections::HashSet, ops::Deref};
use zyheeda_core::prelude::*;

#[serde_model(no_default_deserialize)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct UserInputCombination<const N: usize>(#[serde(with = "array_as_vec")] [RawInput; N]);

impl UserInputCombination<2> {
	pub const ROTATE_DEBUG_NAV_MESH: Self = Self([
		RawInput::KeyCode(KeyCode::ShiftLeft),
		RawInput::KeyCode(KeyCode::F10),
	]);

	pub const ROTATE_DEBUG_COLLIDERS: Self = Self([
		RawInput::KeyCode(KeyCode::ShiftLeft),
		RawInput::KeyCode(KeyCode::F11),
	]);

	pub fn iter(&self) -> std::slice::Iter<'_, RawInput> {
		self.into_iter()
	}

	pub fn as_array(self) -> [RawInput; 2] {
		self.0
	}
}

impl<const N: usize> TryFrom<[RawInput; N]> for UserInputCombination<N> {
	type Error = InputRepeated;

	fn try_from(input: [RawInput; N]) -> Result<Self, Self::Error> {
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
				RawInput::KeyCode(key_code) => TokenItem::from(key_code),
				RawInput::MouseButton(mouse_button) => TokenItem::from(mouse_button),
			})
			.intersperse_with_item(|| TokenItem::Raw(String::from(" + ")));

		Token::from_iter(items)
	}
}

impl<'a, const N: usize> IntoIterator for &'a UserInputCombination<N> {
	type Item = &'a RawInput;
	type IntoIter = std::slice::Iter<'a, RawInput>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.iter()
	}
}

impl<const N: usize> Deref for UserInputCombination<N> {
	type Target = [RawInput];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

#[derive(Debug, PartialEq)]
pub struct InputRepeated(RawInput);

#[cfg(test)]
mod tests {
	use super::*;
	use test_case::test_case;

	#[test]
	fn map_values() -> Result<(), InputRepeated> {
		let UserInputCombination(c) = UserInputCombination::try_from([
			RawInput::KeyCode(KeyCode::KeyA),
			RawInput::MouseButton(MouseButton::Right),
		])?;

		assert_eq!(
			[
				RawInput::KeyCode(KeyCode::KeyA),
				RawInput::MouseButton(MouseButton::Right)
			],
			c
		);
		Ok(())
	}

	#[test]
	fn reject_repeating() {
		let c = UserInputCombination::try_from([
			RawInput::KeyCode(KeyCode::KeyA),
			RawInput::KeyCode(KeyCode::KeyA),
		]);

		assert_eq!(Err(InputRepeated(RawInput::KeyCode(KeyCode::KeyA))), c);
	}

	#[test_case(UserInputCombination::ROTATE_DEBUG_NAV_MESH; "rotate debug nav mesh")]
	#[test_case(UserInputCombination::ROTATE_DEBUG_COLLIDERS; "rotate debug colliders")]
	fn const_not_repeating<const N: usize>(UserInputCombination(inputs): UserInputCombination<N>) {
		assert!(UserInputCombination::try_from(inputs).is_ok());
	}

	#[test]
	fn token() -> Result<(), InputRepeated> {
		let combo = UserInputCombination::try_from([
			RawInput::KeyCode(KeyCode::KeyA),
			RawInput::MouseButton(MouseButton::Right),
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
