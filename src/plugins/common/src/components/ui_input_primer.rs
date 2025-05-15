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
	Primed {
		released: bool,
	},
	JustPressed,
	Pressed,
	JustReleased,
}

pub(crate) trait UiInputStateTransition: Sized {
	fn get_new_state(&self, interaction: &Interaction) -> Option<UiInputState>;
	fn set_state(&mut self, state: UiInputState);
}

impl UiInputStateTransition for UiInputPrimer {
	fn get_new_state(&self, interaction: &Interaction) -> Option<UiInputState> {
		if interaction == &Interaction::Pressed {
			return match self.state {
				UiInputState::None => Some(UiInputState::Primed { released: false }),
				UiInputState::Primed { released: true } => Some(UiInputState::JustPressed),
				UiInputState::JustPressed => Some(UiInputState::Pressed),
				UiInputState::JustReleased => Some(UiInputState::Primed { released: false }),
				_ => None,
			};
		}

		if interaction == &Interaction::None {
			return match self.state {
				UiInputState::Primed { .. } => Some(UiInputState::Primed { released: true }),
				UiInputState::JustPressed => Some(UiInputState::JustReleased),
				UiInputState::Pressed => Some(UiInputState::JustReleased),
				UiInputState::JustReleased => Some(UiInputState::None),
				_ => None,
			};
		}

		None
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
			(None, None, Some(UiInputState::Primed { released: false })),
			(
				input.get_new_state(&Interaction::None),
				input.get_new_state(&Interaction::Hovered),
				input.get_new_state(&Interaction::Pressed),
			),
		);
	}

	#[test]
	fn definition_for_primed_before_release() {
		let input = UiInputPrimer {
			key: UserInput::from(KeyCode::KeyA),
			state: UiInputState::Primed { released: false },
		};

		assert_eq!(
			(Some(UiInputState::Primed { released: true }), None, None),
			(
				input.get_new_state(&Interaction::None),
				input.get_new_state(&Interaction::Hovered),
				input.get_new_state(&Interaction::Pressed),
			),
		);
	}

	#[test]
	fn definition_for_primed_after_release() {
		let input = UiInputPrimer {
			key: UserInput::from(KeyCode::KeyA),
			state: UiInputState::Primed { released: true },
		};

		assert_eq!(
			(
				Some(UiInputState::Primed { released: true }),
				None,
				Some(UiInputState::JustPressed),
			),
			(
				input.get_new_state(&Interaction::None),
				input.get_new_state(&Interaction::Hovered),
				input.get_new_state(&Interaction::Pressed),
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
				Some(UiInputState::JustReleased),
				None,
				Some(UiInputState::Pressed),
			),
			(
				input.get_new_state(&Interaction::None),
				input.get_new_state(&Interaction::Hovered),
				input.get_new_state(&Interaction::Pressed),
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
			(Some(UiInputState::JustReleased), None, None),
			(
				input.get_new_state(&Interaction::None),
				input.get_new_state(&Interaction::Hovered),
				input.get_new_state(&Interaction::Pressed),
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
				None,
				Some(UiInputState::Primed { released: false }),
			),
			(
				input.get_new_state(&Interaction::None),
				input.get_new_state(&Interaction::Hovered),
				input.get_new_state(&Interaction::Pressed),
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

	#[test_case(UiInputState::Primed {released: false}, true; "true if primed not released")]
	#[test_case(UiInputState::Primed {released: true}, true; "true if primed released")]
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
