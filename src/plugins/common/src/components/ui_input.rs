use crate::tools::action_key::user_input::UserInput;
use bevy::prelude::*;

#[derive(Component, Debug, PartialEq, Clone, Copy)]
#[require(Interaction)]
pub struct UiInput {
	pub(crate) key: UserInput,
	pub(crate) state: UiInputState,
}

impl UiInput {
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

impl UiInputStateTransition for UiInput {
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn definition_for_none() {
		let input = UiInput {
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
		let input = UiInput {
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
		let input = UiInput {
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
		let input = UiInput {
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
		let input = UiInput {
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
		let input = UiInput {
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
		let mut input = UiInput {
			key: UserInput::from(KeyCode::KeyA),
			state: UiInputState::None,
		};

		input.set_state(UiInputState::Pressed);

		assert_eq!(
			UiInput {
				key: UserInput::from(KeyCode::KeyA),
				state: UiInputState::Pressed,
			},
			input
		);
	}
}
