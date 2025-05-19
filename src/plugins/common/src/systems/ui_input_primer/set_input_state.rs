use crate::components::ui_input_primer::{LeftMouse, MouseUiInteraction, UiInputStateTransition};
use bevy::{ecs::component::Mutable, prelude::*};

impl<T> SetInputState for T where T: UiInputStateTransition + Component<Mutability = Mutable> {}

pub(crate) trait SetInputState:
	UiInputStateTransition + Component<Mutability = Mutable>
{
	fn set_input_state(
		mut inputs: Query<(&mut Self, Ref<Interaction>)>,
		mouse: Res<ButtonInput<MouseButton>>,
	) {
		let mouse_changed = mouse.is_changed();

		for (mut input, interaction) in &mut inputs {
			if !mouse_changed && !interaction.is_changed() {
				continue;
			}

			let interaction = match *interaction {
				Interaction::Pressed => MouseUiInteraction::Pressed,
				Interaction::Hovered => MouseUiInteraction::Hovered,
				Interaction::None if mouse.just_pressed(MouseButton::Left) => {
					MouseUiInteraction::None(LeftMouse::JustPressed)
				}
				Interaction::None if mouse.just_released(MouseButton::Left) => {
					MouseUiInteraction::None(LeftMouse::JustReleased)
				}
				Interaction::None => MouseUiInteraction::None(LeftMouse::None),
			};

			let Some(state) = input.get_new_state(&interaction) else {
				continue;
			};

			input.set_state(state);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::ui_input_primer::{LeftMouse, UiInputState},
		test_tools::utils::SingleThreadedApp,
	};
	use common::traits::nested_mock::NestedMocks;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::ops::DerefMut;

	#[derive(Component, NestedMocks)]
	struct _Input {
		mock: Mock_Input,
	}

	#[automock]
	impl UiInputStateTransition for _Input {
		fn get_new_state(&self, interaction: &MouseUiInteraction) -> Option<UiInputState> {
			self.mock.get_new_state(interaction)
		}

		fn set_state(&mut self, state: UiInputState) {
			self.mock.set_state(state);
		}
	}

	fn setup(mouse: ButtonInput<MouseButton>) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(mouse);
		app.add_systems(Update, _Input::set_input_state);

		app
	}

	#[test]
	fn set_state_from_mouse_click() {
		let mut mouse = ButtonInput::default();
		mouse.press(MouseButton::Left);
		let mut app = setup(mouse);
		let input = _Input::new().with_mock(|mock| {
			mock.expect_get_new_state()
				.times(1)
				.with(eq(MouseUiInteraction::None(LeftMouse::JustPressed)))
				.return_const(Some(UiInputState::Pressed));
			mock.expect_set_state()
				.times(1)
				.with(eq(UiInputState::Pressed))
				.return_const(());
		});
		app.world_mut().spawn((input, Interaction::None));

		app.update();
	}

	#[test]
	fn set_state_from_none_when_not_just_pressed() {
		let mut mouse = ButtonInput::default();
		mouse.press(MouseButton::Left);
		mouse.clear();
		let mut app = setup(mouse);
		let input = _Input::new().with_mock(|mock| {
			mock.expect_get_new_state()
				.times(1)
				.with(eq(MouseUiInteraction::None(LeftMouse::None)))
				.return_const(None);
			mock.expect_set_state().return_const(());
		});
		app.world_mut().spawn((input, Interaction::None));

		app.update();
	}

	#[test]
	fn set_state_from_mouse_release() {
		let mut mouse = ButtonInput::default();
		mouse.press(MouseButton::Left);
		mouse.clear();
		mouse.release(MouseButton::Left);
		let mut app = setup(mouse);
		let input = _Input::new().with_mock(|mock| {
			mock.expect_get_new_state()
				.times(1)
				.with(eq(MouseUiInteraction::None(LeftMouse::JustReleased)))
				.return_const(None);
			mock.expect_set_state().return_const(());
		});
		app.world_mut().spawn((input, Interaction::None));

		app.update();
	}

	#[test]
	fn set_state_from_none_when_not_just_released() {
		let mut mouse = ButtonInput::default();
		mouse.press(MouseButton::Left);
		mouse.release(MouseButton::Left);
		mouse.clear();
		let mut app = setup(mouse);
		let input = _Input::new().with_mock(|mock| {
			mock.expect_get_new_state()
				.times(1)
				.with(eq(MouseUiInteraction::None(LeftMouse::None)))
				.return_const(None);
			mock.expect_set_state().return_const(());
		});
		app.world_mut().spawn((input, Interaction::None));

		app.update();
	}

	#[test]
	fn set_state_from_interaction_when_pressed() {
		let mut mouse = ButtonInput::default();
		mouse.press(MouseButton::Left);
		mouse.release(MouseButton::Left);
		let mut app = setup(mouse);
		let input = _Input::new().with_mock(|mock| {
			mock.expect_get_new_state()
				.times(1)
				.with(eq(MouseUiInteraction::Pressed))
				.return_const(None);
			mock.expect_set_state().return_const(());
		});
		app.world_mut().spawn((input, Interaction::Pressed));

		app.update();
	}

	#[test]
	fn set_state_from_interaction_when_hovered() {
		let mut mouse = ButtonInput::default();
		mouse.press(MouseButton::Left);
		mouse.release(MouseButton::Left);
		let mut app = setup(mouse);
		let input = _Input::new().with_mock(|mock| {
			mock.expect_get_new_state()
				.times(1)
				.with(eq(MouseUiInteraction::Hovered))
				.return_const(None);
			mock.expect_set_state().return_const(());
		});
		app.world_mut().spawn((input, Interaction::Hovered));

		app.update();
	}

	#[test]
	fn only_set_if_input_changed() {
		let mut mouse = ButtonInput::default();
		mouse.press(MouseButton::Left);
		let mut app = setup(mouse);
		let input = _Input::new().with_mock(|mock| {
			mock.expect_get_new_state()
				.return_const(Some(UiInputState::Pressed));
			mock.expect_set_state()
				.times(2)
				.with(eq(UiInputState::Pressed))
				.return_const(());
		});
		app.world_mut().spawn((input, Interaction::None));

		app.update();
		app.update();
		app.world_mut()
			.resource_mut::<ButtonInput<MouseButton>>()
			.deref_mut();
		app.update();
	}

	#[test]
	fn also_set_if_interaction_changed() {
		let mut mouse = ButtonInput::default();
		mouse.press(MouseButton::Left);
		let mut app = setup(mouse);
		let input = _Input::new().with_mock(|mock| {
			mock.expect_get_new_state()
				.return_const(Some(UiInputState::Pressed));
			mock.expect_set_state()
				.times(2)
				.with(eq(UiInputState::Pressed))
				.return_const(());
		});
		let entity = app.world_mut().spawn((input, Interaction::None)).id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<Interaction>()
			.as_deref_mut();
		app.update();
	}
}
