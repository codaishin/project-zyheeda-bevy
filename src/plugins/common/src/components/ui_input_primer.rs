use crate::{
	tools::action_key::user_input::UserInput,
	traits::accessors::get::{GetProperty, Property},
};
use bevy::prelude::*;

/// Alters user input behavior for the associated UI button.
///
/// When this component is attached to a button:
/// - Pressing the button with the **left mouse button** will *prime* the associated `UserInput`
///   without updating it in `ButtonInput<UserInput>`.
/// - While the input is primed, further state changes to `ButtonInput<UserInput>` for that key are
///   suppressed.
/// - While the button is hovered, further state changes to `ButtonInput<UserInput>` for
///   `UserInput::Mouse(MouseButton::Left)` are suppressed
/// - Pressing and releasing the **left mouse button** a second time will finalize the input,
///   updating `ButtonInput<UserInput>` as if the corresponding `UserInput` key had been directly
///   pressed and released.
#[derive(Component, Debug, PartialEq, Clone, Copy)]
#[require(Interaction)]
pub struct UiInputPrimer {
	pub(crate) key: UserInput,
	pub(crate) state: UiInputState,
}

impl From<UserInput> for UiInputPrimer {
	fn from(key: UserInput) -> Self {
		Self {
			key,
			state: UiInputState::None,
		}
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct IsPrimed(pub bool);

impl From<UiInputPrimer> for IsPrimed {
	fn from(UiInputPrimer { state, .. }: UiInputPrimer) -> Self {
		if state == UiInputState::Primed {
			return Self(true);
		}

		Self(false)
	}
}

impl GetProperty<IsPrimed> for UiInputPrimer {
	fn get_property(&self) -> bool {
		matches!(self.state, UiInputState::Primed)
	}
}

impl GetProperty<UserInput> for UiInputPrimer {
	fn get_property(&self) -> UserInput {
		self.key
	}
}

impl GetProperty<JustChangedInput> for UiInputPrimer {
	fn get_property(&self) -> JustChangedInput {
		match self.state {
			UiInputState::JustPressed => JustChangedInput::JustPressed(self.key),
			UiInputState::JustReleased => JustChangedInput::JustReleased(self.key),
			_ => JustChangedInput::None,
		}
	}
}

impl Property for IsPrimed {
	type TValue<'a> = bool;
}

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub(crate) enum UiInputState {
	#[default]
	None,
	Hovered,
	Primed,
	JustPressed,
	Pressed,
	JustReleased,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum MouseUiInteraction {
	None(LeftMouse),
	Hovered,
	Pressed,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum LeftMouse {
	None,
	JustPressed,
	JustReleased,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum JustChangedInput {
	None,
	JustPressed(UserInput),
	JustReleased(UserInput),
}

impl Property for JustChangedInput {
	type TValue<'a> = Self;
}

pub(crate) trait UiInputStateTransition: Sized {
	fn get_new_state(&self, interaction: &MouseUiInteraction) -> Option<UiInputState>;
	fn set_state(&mut self, state: UiInputState);
}

impl UiInputStateTransition for UiInputPrimer {
	fn get_new_state(&self, interaction: &MouseUiInteraction) -> Option<UiInputState> {
		match (self.state, interaction) {
			(UiInputState::None, MouseUiInteraction::Hovered) => Some(UiInputState::Hovered),
			(UiInputState::None | UiInputState::JustReleased, MouseUiInteraction::Pressed) => {
				Some(UiInputState::Primed)
			}
			(UiInputState::Primed, MouseUiInteraction::None(LeftMouse::JustPressed)) => {
				Some(UiInputState::JustPressed)
			}
			(
				UiInputState::JustPressed | UiInputState::Pressed,
				MouseUiInteraction::None(LeftMouse::JustReleased),
			) => Some(UiInputState::JustReleased),
			(UiInputState::Hovered, MouseUiInteraction::Pressed) => Some(UiInputState::Primed),
			(UiInputState::Hovered, MouseUiInteraction::None(_)) => Some(UiInputState::None),
			(UiInputState::JustPressed, _) => Some(UiInputState::Pressed),
			(UiInputState::JustReleased, _) => Some(UiInputState::None),
			_ => None,
		}
	}

	fn set_state(&mut self, state: UiInputState) {
		self.state = state
	}
}

pub(crate) trait KeyPrimed {
	fn key_primed(&self, key: &UserInput) -> bool;
}

impl KeyPrimed for UiInputPrimer {
	fn key_primed(&self, key: &UserInput) -> bool {
		match self.state {
			UiInputState::Hovered => key == &UserInput::from(MouseButton::Left),
			UiInputState::Primed => key == &self.key || key == &UserInput::from(MouseButton::Left),
			UiInputState::JustPressed => key == &UserInput::from(MouseButton::Left),
			_ => false,
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::traits::accessors::get::DynProperty;

	use super::*;
	use test_case::test_case;

	#[test]
	fn get_is_primed_true() {
		let input = UiInputPrimer {
			key: UserInput::from(KeyCode::ArrowLeft),
			state: UiInputState::Primed,
		};

		assert_eq!(IsPrimed(true), IsPrimed::from(input))
	}

	#[test_case(UiInputState::None; "none")]
	#[test_case(UiInputState::JustPressed; "just pressed")]
	#[test_case(UiInputState::Pressed; "pressed")]
	#[test_case(UiInputState::JustReleased; "just released")]
	fn get_is_primed_false(state: UiInputState) {
		let input = UiInputPrimer {
			key: UserInput::from(KeyCode::ArrowLeft),
			state,
		};

		assert_eq!(IsPrimed(false), IsPrimed::from(input))
	}

	#[test]
	fn get_user_input() {
		let input = UiInputPrimer {
			key: UserInput::from(KeyCode::ArrowLeft),
			state: UiInputState::Primed,
		};

		assert_eq!(
			UserInput::from(KeyCode::ArrowLeft),
			input.dyn_property::<UserInput>(),
		)
	}

	#[test]
	fn get_input_none() {
		let input = UiInputPrimer {
			key: UserInput::from(KeyCode::ArrowLeft),
			state: UiInputState::None,
		};

		assert_eq!(
			JustChangedInput::None,
			input.dyn_property::<JustChangedInput>(),
		);
	}

	#[test]
	fn get_input_just_pressed() {
		let input = UiInputPrimer {
			key: UserInput::from(KeyCode::ArrowLeft),
			state: UiInputState::JustPressed,
		};

		assert_eq!(
			JustChangedInput::JustPressed(UserInput::from(KeyCode::ArrowLeft)),
			input.dyn_property::<JustChangedInput>(),
		);
	}

	#[test]
	fn get_input_just_released() {
		let input = UiInputPrimer {
			key: UserInput::from(KeyCode::ArrowLeft),
			state: UiInputState::JustReleased,
		};

		assert_eq!(
			JustChangedInput::JustReleased(UserInput::from(KeyCode::ArrowLeft)),
			input.dyn_property::<JustChangedInput>(),
		);
	}

	#[test]
	fn definition_for_none() {
		let input = UiInputPrimer {
			key: UserInput::from(KeyCode::KeyA),
			state: UiInputState::None,
		};

		assert_eq!(
			(
				None,
				None,
				None,
				Some(UiInputState::Hovered),
				Some(UiInputState::Primed)
			),
			(
				input.get_new_state(&MouseUiInteraction::None(LeftMouse::None)),
				input.get_new_state(&MouseUiInteraction::None(LeftMouse::JustPressed)),
				input.get_new_state(&MouseUiInteraction::None(LeftMouse::JustReleased)),
				input.get_new_state(&MouseUiInteraction::Hovered),
				input.get_new_state(&MouseUiInteraction::Pressed),
			),
		);
	}

	#[test]
	fn definition_for_hovered() {
		let input = UiInputPrimer {
			key: UserInput::from(KeyCode::KeyA),
			state: UiInputState::Hovered,
		};

		assert_eq!(
			(
				Some(UiInputState::None),
				Some(UiInputState::None),
				Some(UiInputState::None),
				None,
				Some(UiInputState::Primed)
			),
			(
				input.get_new_state(&MouseUiInteraction::None(LeftMouse::None)),
				input.get_new_state(&MouseUiInteraction::None(LeftMouse::JustPressed)),
				input.get_new_state(&MouseUiInteraction::None(LeftMouse::JustReleased)),
				input.get_new_state(&MouseUiInteraction::Hovered),
				input.get_new_state(&MouseUiInteraction::Pressed),
			),
		);
	}

	#[test]
	fn definition_for_primed() {
		let input = UiInputPrimer {
			key: UserInput::from(KeyCode::KeyA),
			state: UiInputState::Primed,
		};

		assert_eq!(
			(None, Some(UiInputState::JustPressed), None, None, None),
			(
				input.get_new_state(&MouseUiInteraction::None(LeftMouse::None)),
				input.get_new_state(&MouseUiInteraction::None(LeftMouse::JustPressed)),
				input.get_new_state(&MouseUiInteraction::None(LeftMouse::JustReleased)),
				input.get_new_state(&MouseUiInteraction::Hovered),
				input.get_new_state(&MouseUiInteraction::Pressed),
			),
		);
	}

	#[test]
	fn definition_for_just_pressed() {
		let input = UiInputPrimer {
			key: UserInput::from(KeyCode::KeyA),
			state: UiInputState::JustPressed,
		};

		assert_eq!(
			(
				Some(UiInputState::Pressed),
				Some(UiInputState::Pressed),
				Some(UiInputState::JustReleased),
				Some(UiInputState::Pressed),
				Some(UiInputState::Pressed),
			),
			(
				input.get_new_state(&MouseUiInteraction::None(LeftMouse::None)),
				input.get_new_state(&MouseUiInteraction::None(LeftMouse::JustPressed)),
				input.get_new_state(&MouseUiInteraction::None(LeftMouse::JustReleased)),
				input.get_new_state(&MouseUiInteraction::Hovered),
				input.get_new_state(&MouseUiInteraction::Pressed),
			),
		);
	}

	#[test]
	fn definition_for_pressed() {
		let input = UiInputPrimer {
			key: UserInput::from(KeyCode::KeyA),
			state: UiInputState::Pressed,
		};

		assert_eq!(
			(None, None, Some(UiInputState::JustReleased), None, None),
			(
				input.get_new_state(&MouseUiInteraction::None(LeftMouse::None)),
				input.get_new_state(&MouseUiInteraction::None(LeftMouse::JustPressed)),
				input.get_new_state(&MouseUiInteraction::None(LeftMouse::JustReleased)),
				input.get_new_state(&MouseUiInteraction::Hovered),
				input.get_new_state(&MouseUiInteraction::Pressed),
			),
		);
	}

	#[test]
	fn definition_for_just_released() {
		let input = UiInputPrimer {
			key: UserInput::from(KeyCode::KeyA),
			state: UiInputState::JustReleased,
		};

		assert_eq!(
			(
				Some(UiInputState::None),
				Some(UiInputState::None),
				Some(UiInputState::None),
				Some(UiInputState::None),
				Some(UiInputState::Primed),
			),
			(
				input.get_new_state(&MouseUiInteraction::None(LeftMouse::None)),
				input.get_new_state(&MouseUiInteraction::None(LeftMouse::JustPressed)),
				input.get_new_state(&MouseUiInteraction::None(LeftMouse::JustReleased)),
				input.get_new_state(&MouseUiInteraction::Hovered),
				input.get_new_state(&MouseUiInteraction::Pressed),
			),
		);
	}

	#[test]
	fn set_new_state() {
		let mut input = UiInputPrimer {
			key: UserInput::from(KeyCode::KeyA),
			state: UiInputState::None,
		};

		input.set_state(UiInputState::Pressed);

		assert_eq!(
			UiInputPrimer {
				key: UserInput::from(KeyCode::KeyA),
				state: UiInputState::Pressed,
			},
			input
		);
	}

	#[test_case(UiInputState::Primed, true; "true if primed")]
	#[test_case(UiInputState::None, false; "none not primed")]
	#[test_case(UiInputState::JustPressed, false; "false if just pressed")]
	#[test_case(UiInputState::Pressed, false; "false if pressed")]
	#[test_case(UiInputState::JustReleased, false; "false if just released")]
	fn primed_with_matching_key(state: UiInputState, is_primed: bool) {
		let input = UiInputPrimer {
			key: UserInput::from(KeyCode::KeyA),
			state,
		};

		assert_eq!(is_primed, input.key_primed(&UserInput::from(KeyCode::KeyA)));
	}

	#[test]
	fn is_not_primed_for_different_key() {
		let input = UiInputPrimer::from(UserInput::from(KeyCode::KeyA));

		assert!(!input.key_primed(&UserInput::from(KeyCode::KeyB)));
	}

	#[test]
	fn is_primed_for_left_mouse_when_hovered() {
		let input = UiInputPrimer {
			key: UserInput::from(KeyCode::KeyA),
			state: UiInputState::Hovered,
		};

		assert!(input.key_primed(&UserInput::from(MouseButton::Left)));
	}

	#[test]
	fn is_primed_for_left_mouse_when_primed() {
		let input = UiInputPrimer {
			key: UserInput::from(KeyCode::KeyA),
			state: UiInputState::Primed,
		};

		assert!(input.key_primed(&UserInput::from(MouseButton::Left)));
	}

	#[test]
	fn is_primed_for_left_mouse_when_just_pressed() {
		let input = UiInputPrimer {
			key: UserInput::from(KeyCode::KeyA),
			state: UiInputState::JustPressed,
		};

		assert!(input.key_primed(&UserInput::from(MouseButton::Left)));
	}
}
