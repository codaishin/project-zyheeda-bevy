use bevy::{prelude::*, state::state::FreelyMutableState};
use common::{
	tools::keys::user_input::UserInput,
	traits::{key_mappings::JustPressed, states::PlayState},
};

pub(crate) fn set_state_from_input<TState, TMenuState, TKeyMap>(
	keys: Res<ButtonInput<UserInput>>,
	key_map: Res<TKeyMap>,
	current_state: Res<State<TState>>,
	mut next_state: ResMut<NextState<TState>>,
) where
	TState: States + FreelyMutableState + PlayState + From<TMenuState>,
	TKeyMap: JustPressed<TMenuState> + Resource,
{
	let current = current_state.get();

	for just_pressed in key_map.just_pressed(&keys) {
		let target_state = match TState::from(just_pressed) {
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
	use common::{test_tools::utils::SingleThreadedApp, traits::nested_mock::NestedMocks};
	use macros::NestedMocks;
	use mockall::automock;

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

	#[derive(Resource, NestedMocks)]
	struct _Map {
		mock: Mock_Map,
	}

	#[automock]
	impl JustPressed<_Menu> for _Map {
		fn just_pressed(&self, input: &ButtonInput<UserInput>) -> impl Iterator<Item = _Menu> {
			self.mock.just_pressed(input)
		}
	}

	fn setup(map: _Map) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(StatesPlugin);
		app.init_state::<_State>();
		app.insert_resource(map);
		app.init_resource::<ButtonInput<UserInput>>();
		app.add_systems(Update, set_state_from_input::<_State, _Menu, _Map>);

		app
	}

	#[test]
	fn set_a_on_just_pressed() {
		let mut app = setup(_Map::new().with_mock(|mock| {
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
		let mut app = setup(_Map::new().with_mock(|mock| {
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
		let mut app = setup(_Map::new().with_mock(|mock| {
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
		let mut app = setup(_Map::new().with_mock(|mock| {
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
