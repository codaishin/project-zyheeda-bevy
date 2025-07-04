use crate::{
	components::ui_input_primer::JustChangedInput,
	tools::action_key::user_input::UserInput,
};
use bevy::prelude::*;

impl<T> ApplyInput for T
where
	T: Component,
	for<'a> JustChangedInput: From<&'a Self>,
{
}

pub(crate) trait ApplyInput: Component + Sized
where
	for<'a> JustChangedInput: From<&'a Self>,
{
	fn apply_input(
		mut input: ResMut<ButtonInput<UserInput>>,
		primers: Query<&Self, Changed<Self>>,
	) {
		for primer in &primers {
			match JustChangedInput::from(primer) {
				JustChangedInput::JustPressed(user_input) => {
					input.press(user_input);
				}
				JustChangedInput::JustReleased(user_input) => {
					input.release(user_input);
				}
				JustChangedInput::None => {}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::tools::action_key::user_input::UserInput;
	use testing::SingleThreadedApp;

	#[derive(Component)]
	struct _Input(JustChangedInput);

	impl From<&_Input> for JustChangedInput {
		fn from(_Input(input): &_Input) -> Self {
			*input
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<ButtonInput<UserInput>>();
		app.add_systems(
			Update,
			(
				|mut input: ResMut<ButtonInput<UserInput>>| {
					input.clear();
				},
				_Input::apply_input,
			)
				.chain(),
		);

		app
	}

	#[test]
	fn set_just_pressed() {
		let mut app = setup();
		app.world_mut()
			.spawn(_Input(JustChangedInput::JustPressed(UserInput::from(
				KeyCode::AltLeft,
			))));

		app.update();

		let input = app.world().resource::<ButtonInput<UserInput>>();
		assert_eq!(
			(vec![&UserInput::from(KeyCode::AltLeft)], vec![]),
			(
				input.get_just_pressed().collect::<Vec<_>>(),
				input.get_just_released().collect::<Vec<_>>(),
			)
		);
	}

	#[test]
	fn set_just_released() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(_Input(JustChangedInput::JustPressed(UserInput::from(
				KeyCode::AltLeft,
			))))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<_Input>()
			.unwrap()
			.0 = JustChangedInput::JustReleased(UserInput::from(KeyCode::AltLeft));
		app.update();

		let input = app.world().resource::<ButtonInput<UserInput>>();
		assert_eq!(
			(vec![], vec![&UserInput::from(KeyCode::AltLeft)]),
			(
				input.get_just_pressed().collect::<Vec<_>>(),
				input.get_just_released().collect::<Vec<_>>(),
			)
		);
	}

	#[test]
	fn do_not_set_twice() {
		let mut app = setup();
		app.world_mut()
			.spawn(_Input(JustChangedInput::JustPressed(UserInput::from(
				KeyCode::AltLeft,
			))));

		app.update();
		app.world_mut()
			.resource_mut::<ButtonInput<UserInput>>()
			.reset_all();
		app.update();

		let input = app.world().resource::<ButtonInput<UserInput>>();
		assert_eq!(
			(vec![], vec![]),
			(
				input.get_just_pressed().collect::<Vec<_>>(),
				input.get_just_released().collect::<Vec<_>>(),
			)
		);
	}

	#[test]
	fn set_again_if_input_changed() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(_Input(JustChangedInput::JustPressed(UserInput::from(
				KeyCode::AltLeft,
			))))
			.id();

		app.update();
		app.world_mut()
			.resource_mut::<ButtonInput<UserInput>>()
			.reset_all();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<_Input>()
			.as_deref_mut();
		app.update();

		let input = app.world().resource::<ButtonInput<UserInput>>();
		assert_eq!(
			(vec![&UserInput::from(KeyCode::AltLeft)], vec![]),
			(
				input.get_just_pressed().collect::<Vec<_>>(),
				input.get_just_released().collect::<Vec<_>>(),
			)
		);
	}
}
