use bevy::{
	ecs::system::{Res, ResMut},
	input::{keyboard::KeyCode, ButtonInput},
	state::state::{FreelyMutableState, NextState, State, States},
};
use common::traits::{iteration::IterFinite, states::PlayState};

pub(crate) fn set_state_from_input<
	TState: States + FreelyMutableState + PlayState + IterFinite + Copy,
>(
	keys: Res<ButtonInput<KeyCode>>,
	current_state: Res<State<TState>>,
	mut next_state: ResMut<NextState<TState>>,
) where
	KeyCode: TryFrom<TState>,
{
	let get_new_state = |state| get_new_state(&keys, current_state.get(), state);
	let Some(new_state) = TState::iterator().find_map(get_new_state) else {
		return;
	};

	next_state.set(new_state);
}

fn get_new_state<TState: States + PlayState + IterFinite + Copy>(
	keys: &Res<ButtonInput<KeyCode>>,
	current_state: &TState,
	state: TState,
) -> Option<TState>
where
	KeyCode: TryFrom<TState>,
{
	let Ok(key) = KeyCode::try_from(state) else {
		return None;
	};

	if !keys.just_pressed(key) {
		return None;
	}

	match current_state == &state {
		true => Some(TState::play_state()),
		false => Some(state),
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		state::app::{AppExtStates, StatesPlugin},
	};
	use common::{test_tools::utils::SingleThreadedApp, traits::iteration::Iter};

	#[derive(Default, Debug, PartialEq, States, Hash, Eq, Clone, Copy)]
	enum _State {
		#[default]
		Default,
		Play,
		A,
		B,
	}

	impl PlayState for _State {
		fn play_state() -> Self {
			_State::Play
		}
	}

	impl IterFinite for _State {
		fn iterator() -> Iter<Self> {
			Iter(Some(_State::Default))
		}

		fn next(current: &Iter<Self>) -> Option<Self> {
			match current.0? {
				_State::Default => Some(_State::Play),
				_State::Play => Some(_State::A),
				_State::A => Some(_State::B),
				_State::B => None,
			}
		}
	}

	impl TryFrom<_State> for KeyCode {
		type Error = ();

		fn try_from(value: _State) -> Result<Self, Self::Error> {
			match value {
				_State::Default => Err(()),
				_State::Play => Err(()),
				_State::A => Ok(KeyCode::KeyA),
				_State::B => Ok(KeyCode::KeyB),
			}
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_plugins(StatesPlugin);
		app.init_state::<_State>();
		app.init_resource::<ButtonInput<KeyCode>>();
		app.add_systems(Update, set_state_from_input::<_State>);

		app
	}

	#[test]
	fn set_a_on_just_pressed() {
		let mut app = setup();
		let mut input = app
			.world_mut()
			.get_resource_mut::<ButtonInput<KeyCode>>()
			.unwrap();
		input.press(KeyCode::KeyA);

		app.update();
		app.update();

		let state = app.world().get_resource::<State<_State>>().unwrap();
		assert_eq!(&_State::A, state.get());
	}

	#[test]
	fn do_not_set_when_not_just_pressed() {
		let mut app = setup();
		let mut input = app
			.world_mut()
			.get_resource_mut::<ButtonInput<KeyCode>>()
			.unwrap();
		input.press(KeyCode::KeyA);
		input.clear_just_pressed(KeyCode::KeyA);

		app.update();
		app.update();

		let state = app.world().get_resource::<State<_State>>().unwrap();
		assert_eq!(&_State::Default, state.get());
	}

	#[test]
	fn set_to_play_on_a_if_already_a() {
		let mut app = setup();
		let mut input = app
			.world_mut()
			.get_resource_mut::<ButtonInput<KeyCode>>()
			.unwrap();
		input.press(KeyCode::KeyA);
		app.insert_resource(State::new(_State::A));

		app.update();
		app.update();

		let state = app.world().get_resource::<State<_State>>().unwrap();
		assert_eq!(&_State::Play, state.get());
	}
}
