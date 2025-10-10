use bevy::{
	ecs::system::{StaticSystemParam, SystemParam, SystemParamItem},
	prelude::*,
	state::state::FreelyMutableState,
};
use common::{
	tools::action_key::ActionKey,
	traits::{
		handles_input::{GetInputState, InputState},
		iteration::IterFinite,
		states::PlayState,
	},
};

pub(crate) fn set_state_from_input<TState, TMenuState, TInput>(
	input: StaticSystemParam<TInput>,
	current_state: Res<State<TState>>,
	mut next_state: ResMut<NextState<TState>>,
) where
	TState: States + FreelyMutableState + PlayState + From<TMenuState>,
	TMenuState: Copy + IterFinite + Into<ActionKey> + 'static,
	TInput: SystemParam,
	for<'w, 's> SystemParamItem<'w, 's, TInput>: GetInputState,
{
	let current = current_state.get();

	for state in TMenuState::iterator() {
		let InputState::Pressed { just_now: true } = input.get_input_state(state) else {
			continue;
		};
		let target_state = match TState::from(state) {
			just_pressed if &just_pressed == current => TState::play_state(),
			just_pressed => just_pressed,
		};
		next_state.set(target_state);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		state::app::{AppExtStates, StatesPlugin},
	};
	use common::tools::action_key::user_input::UserInput;
	use macros::NestedMocks;
	use mockall::automock;
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Default, Debug, PartialEq, States, Hash, Eq, Clone, Copy)]
	enum _State {
		#[default]
		Default,
		Play,
		Menu(_Menu),
	}

	impl PlayState for _State {
		fn play_state() -> Self {
			_State::Play
		}
	}

	#[derive(Debug, PartialEq, States, Hash, Eq, Clone, Copy)]
	struct _Menu;

	impl From<_Menu> for _State {
		fn from(menu: _Menu) -> Self {
			_State::Menu(menu)
		}
	}

	#[derive(SystemParam)]
	struct _InputParam<'w> {
		input: Res<'w, _Input>,
	}

	impl GetInputState for _InputParam<'_> {
		fn get_input_state<TAction>(&self, action: TAction) -> InputState
		where
			TAction: Into<ActionKey> + 'static,
		{
			self.input.get_input_state(action)
		}
	}

	#[derive(Resource, NestedMocks)]
	struct _Input {
		mock: Mock_Input,
	}

	#[automock]
	impl GetInputState for _Input {
		fn get_input_state<TAction>(&self, action: TAction) -> InputState
		where
			TAction: Into<ActionKey> + 'static,
		{
			self.mock.get_input_state(action)
		}
	}

	fn setup(map: _Input) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(StatesPlugin);
		app.init_state::<_State>();
		app.insert_resource(map);
		app.init_resource::<ButtonInput<UserInput>>();
		app.add_systems(Update, set_state_from_input::<_State, _Menu, _InputParam>);

		app
	}

	#[test]
	fn set_a_on_just_pressed() {
		let mut app = setup(_InputParam::new().with_mock(|mock| {
			mock.expect_just_pressed()
				.returning(|_| Box::new(std::iter::once(_Menu)));
		}));

		app.update();
		app.update();

		let state = app.world().get_resource::<State<_State>>().unwrap();
		assert_eq!(&_State::Menu(_Menu), state.get());
	}

	#[test]
	fn do_not_set_when_not_just_pressed() {
		let mut app = setup(_InputParam::new().with_mock(|mock| {
			mock.expect_just_pressed()
				.returning(|_| Box::new(std::iter::empty()));
		}));

		app.update();
		app.update();

		let state = app.world().get_resource::<State<_State>>().unwrap();
		assert_eq!(&_State::Default, state.get());
	}

	#[test]
	fn set_to_play_on_a_if_already_a() {
		let mut app = setup(_InputParam::new().with_mock(|mock| {
			mock.expect_just_pressed()
				.returning(|_| Box::new(std::iter::once(_Menu)));
		}));
		app.insert_resource(State::new(_State::Menu(_Menu)));

		app.update();
		app.update();

		let state = app.world().get_resource::<State<_State>>().unwrap();
		assert_eq!(&_State::Play, state.get());
	}

	#[test]
	fn call_map_with_correct_input() {
		let mut app = setup(_InputParam::new().with_mock(|mock| {
			mock.expect_just_pressed().times(1).returning(|input| {
				assert_eq!(
					vec![&UserInput::from(KeyCode::ArrowUp)],
					input.get_just_pressed().collect::<Vec<_>>()
				);
				Box::new(std::iter::empty())
			});
		}));
		let mut input = ButtonInput::default();
		input.press(UserInput::KeyCode(KeyCode::ArrowUp));
		app.insert_resource(input);

		app.update();
	}
}
