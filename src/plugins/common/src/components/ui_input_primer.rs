use crate::tools::action_key::user_input::UserInput;
use bevy::prelude::*;

/// Alters user input behavior for the associated UI button.
///
/// When this component is attached to a button:
/// - Pressing the button with the **left mouse button** will *prime* the associated `UserInput`
///   without updating it in `ButtonInput<UserInput>`.
/// - While the input is primed, further state changes to `ButtonInput<UserInput>` for that key are
///   suppressed.
/// - Pressing and releasing the **left mouse button** a second time will finalize the input,
///   updating `ButtonInput<UserInput>` as if the corresponding `UserInput` key had been directly
///   pressed or released.
#[derive(Component, Debug, PartialEq, Clone, Copy)]
#[require(Interaction)]
pub struct UiInputPrimer {
	pub(crate) key: UserInput,
	pub(crate) state: UiInputState,
}

impl UiInputPrimer {
	pub fn new(key: UserInput) -> Self {
		Self {
			key,
			state: UiInputState::default(),
		}
	}
}

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub(crate) enum UiInputState {
	#[default]
	None,
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

pub(crate) trait UiInputStateTransition: Sized {
	fn get_new_state(&self, interaction: &MouseUiInteraction) -> Option<UiInputState>;
	fn set_state(&mut self, state: UiInputState);
}

impl UiInputStateTransition for UiInputPrimer {
	fn get_new_state(&self, interaction: &MouseUiInteraction) -> Option<UiInputState> {
		match (interaction, self.state) {
			// prime
			(MouseUiInteraction::Pressed, UiInputState::None | UiInputState::JustReleased) => {
				Some(UiInputState::Primed)
			}
			// press key
			(MouseUiInteraction::None(LeftMouse::JustPressed), UiInputState::Primed) => {
				Some(UiInputState::JustPressed)
			}
			// release key
			(
				MouseUiInteraction::None(LeftMouse::JustReleased),
				UiInputState::JustPressed | UiInputState::Pressed,
			) => Some(UiInputState::JustReleased),
			// outdated press/release
			(_, UiInputState::JustPressed) => Some(UiInputState::Pressed),
			(_, UiInputState::JustReleased) => Some(UiInputState::None),
			// rest
			_ => None,
		}
	}

	fn set_state(&mut self, state: UiInputState) {
		self.state = state
	}
}

pub(crate) trait IsKey {
	fn is_key(&self, key: &UserInput) -> bool;
}

impl IsKey for UiInputPrimer {
	fn is_key(&self, key: &UserInput) -> bool {
		&self.key == key
	}
}

pub(crate) trait IsPrimed {
	fn is_primed(&self) -> bool;
}

impl IsPrimed for UiInputPrimer {
	fn is_primed(&self) -> bool {
		matches!(self.state, UiInputState::Primed { .. })
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use test_case::test_case;

	#[test]
	fn definition_for_none() {
		let input = UiInputPrimer {
			key: UserInput::from(KeyCode::KeyA),
			state: UiInputState::None,
		};

		assert_eq!(
			(None, None, None, None, Some(UiInputState::Primed)),
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
	fn primed(state: UiInputState, is_primed: bool) {
		let input = UiInputPrimer {
			key: UserInput::from(KeyCode::KeyA),
			state,
		};

		assert_eq!(is_primed, input.is_primed());
	}

	#[test]
	fn is_key_when_matching() {
		let input = UiInputPrimer::new(UserInput::from(KeyCode::KeyA));

		assert!(input.is_key(&UserInput::from(KeyCode::KeyA)));
	}

	#[test]
	fn is_not_key_when_not_matching() {
		let input = UiInputPrimer::new(UserInput::from(KeyCode::KeyB));

		assert!(!input.is_key(&UserInput::from(KeyCode::KeyA)));
	}
}
