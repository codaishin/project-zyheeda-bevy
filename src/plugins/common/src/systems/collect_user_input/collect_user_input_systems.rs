use crate::{
	ApplyInput,
	SetInputState,
	components::ui_input_primer::UiInputPrimer,
	tools::action_key::user_input::UserInput,
};
use bevy::prelude::*;

pub(crate) trait CollectUserInputSystems {
	fn collect_user_input(&mut self) -> &mut Self;
}

impl CollectUserInputSystems for App {
	fn collect_user_input(&mut self) -> &mut Self {
		self.init_resource::<ButtonInput<UserInput>>();
		self.add_systems(
			PreUpdate,
			(
				UiInputPrimer::set_input_state,
				UserInput::clear,
				UserInput::collect::<KeyCode, UiInputPrimer>,
				UserInput::collect::<MouseButton, UiInputPrimer>,
				UiInputPrimer::apply_input,
			)
				.chain()
				.in_set(UserInput::SYSTEM)
				.after(bevy::input::InputSystem),
		);

		self
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{components::ui_input_primer::UiInputState, test_tools::utils::SingleThreadedApp};
	use std::collections::HashSet;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<ButtonInput<KeyCode>>();
		app.init_resource::<ButtonInput<MouseButton>>();
		app.collect_user_input();

		app
	}

	#[test]
	fn collect_input_in_same_frame() {
		let mut app = setup();
		app.world_mut()
			.resource_mut::<ButtonInput<KeyCode>>()
			.press(KeyCode::KeyA);
		app.world_mut()
			.resource_mut::<ButtonInput<MouseButton>>()
			.press(MouseButton::Middle);

		app.update();

		assert_eq!(
			HashSet::from([
				&UserInput::from(KeyCode::KeyA),
				&UserInput::from(MouseButton::Middle),
			]),
			app.world()
				.resource::<ButtonInput<UserInput>>()
				.get_just_pressed()
				.collect::<HashSet<_>>()
		);
	}

	#[test]
	fn clear_before_collection() {
		let mut app = setup();
		app.world_mut()
			.resource_mut::<ButtonInput<KeyCode>>()
			.press(KeyCode::KeyA);
		app.world_mut()
			.resource_mut::<ButtonInput<MouseButton>>()
			.press(MouseButton::Middle);
		app.world_mut()
			.resource_mut::<ButtonInput<UserInput>>()
			.press(UserInput::from(KeyCode::Numpad0));

		app.update();

		assert_eq!(
			HashSet::from([
				&UserInput::from(KeyCode::KeyA),
				&UserInput::from(MouseButton::Middle),
			]),
			app.world()
				.resource::<ButtonInput<UserInput>>()
				.get_just_pressed()
				.collect::<HashSet<_>>()
		);
	}

	#[test]
	fn do_not_update_primed_input() {
		let mut app = setup();
		app.world_mut()
			.resource_mut::<ButtonInput<KeyCode>>()
			.press(KeyCode::KeyA);
		app.world_mut()
			.resource_mut::<ButtonInput<MouseButton>>()
			.press(MouseButton::Middle);
		app.world_mut().spawn(UiInputPrimer {
			key: UserInput::from(KeyCode::KeyA),
			state: UiInputState::Primed,
		});
		app.world_mut().spawn(UiInputPrimer {
			key: UserInput::from(MouseButton::Middle),
			state: UiInputState::Primed,
		});

		app.update();

		assert_eq!(
			HashSet::from([]),
			app.world()
				.resource::<ButtonInput<UserInput>>()
				.get_just_pressed()
				.collect::<HashSet<_>>()
		);
	}

	#[test]
	fn do_not_update_input_that_was_just_primed() {
		let mut app = setup();
		app.world_mut()
			.resource_mut::<ButtonInput<KeyCode>>()
			.press(KeyCode::KeyA);
		app.world_mut()
			.resource_mut::<ButtonInput<MouseButton>>()
			.press(MouseButton::Middle);
		app.world_mut().spawn((
			UiInputPrimer {
				key: UserInput::from(KeyCode::KeyA),
				state: UiInputState::None,
			},
			Interaction::Pressed,
		));
		app.world_mut().spawn((
			UiInputPrimer {
				key: UserInput::from(MouseButton::Middle),
				state: UiInputState::None,
			},
			Interaction::Pressed,
		));

		app.update();

		assert_eq!(
			HashSet::from([]),
			app.world()
				.resource::<ButtonInput<UserInput>>()
				.get_just_pressed()
				.collect::<HashSet<_>>()
		);
	}

	#[test]
	fn do_not_update_input_when_hovering_primer_and_clicking_left_mouse() {
		let mut app = setup();
		app.world_mut()
			.resource_mut::<ButtonInput<MouseButton>>()
			.press(MouseButton::Left);
		app.world_mut().spawn((
			UiInputPrimer {
				key: UserInput::from(KeyCode::KeyA),
				state: UiInputState::None,
			},
			Interaction::Hovered,
		));
		app.world_mut().spawn((
			UiInputPrimer {
				key: UserInput::from(MouseButton::Middle),
				state: UiInputState::None,
			},
			Interaction::Hovered,
		));

		app.update();

		assert_eq!(
			HashSet::from([]),
			app.world()
				.resource::<ButtonInput<UserInput>>()
				.get_just_pressed()
				.collect::<HashSet<_>>()
		);
	}

	#[test]
	fn update_primed_input_when_pressing_left_mouse() {
		let mut app = setup();
		app.world_mut()
			.resource_mut::<ButtonInput<MouseButton>>()
			.press(MouseButton::Left);
		app.world_mut().spawn(UiInputPrimer {
			key: UserInput::from(KeyCode::KeyA),
			state: UiInputState::Primed,
		});
		app.world_mut().spawn(UiInputPrimer {
			key: UserInput::from(MouseButton::Middle),
			state: UiInputState::Primed,
		});

		app.update();

		assert_eq!(
			HashSet::from([
				&UserInput::from(KeyCode::KeyA),
				&UserInput::from(MouseButton::Middle),
			]),
			app.world()
				.resource::<ButtonInput<UserInput>>()
				.get_just_pressed()
				.collect::<HashSet<_>>()
		);
	}
}
