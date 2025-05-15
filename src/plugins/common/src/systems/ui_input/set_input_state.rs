use crate::components::ui_input::UiInputStateTransition;
use bevy::prelude::*;

impl<T> SetInputState for T where T: UiInputStateTransition + Component {}

pub(crate) trait SetInputState: UiInputStateTransition + Component {
	fn set_input_state(mut inputs: Query<(&mut Self, &Interaction)>) {
		for (mut input, interaction) in &mut inputs {
			let Some(state) = input.get_new_state(interaction) else {
				continue;
			};

			input.set_state(state);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{components::ui_input::UiInputState, test_tools::utils::SingleThreadedApp};
	use common::traits::nested_mock::NestedMocks;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

	#[derive(Component, NestedMocks)]
	struct _Input {
		mock: Mock_Input,
	}

	#[automock]
	impl UiInputStateTransition for _Input {
		fn get_new_state(&self, interaction: &Interaction) -> Option<UiInputState> {
			self.mock.get_new_state(interaction)
		}

		fn set_state(&mut self, state: UiInputState) {
			self.mock.set_state(state);
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, _Input::set_input_state);

		app
	}

	#[test]
	fn set_state() {
		let mut app = setup();
		let input = _Input::new().with_mock(|mock| {
			mock.expect_get_new_state()
				.times(1)
				.with(eq(Interaction::Pressed))
				.return_const(Some(UiInputState::Pressed));
			mock.expect_set_state()
				.times(1)
				.with(eq(UiInputState::Pressed))
				.return_const(());
		});
		app.world_mut().spawn((input, Interaction::Pressed));

		app.update();
	}
}
