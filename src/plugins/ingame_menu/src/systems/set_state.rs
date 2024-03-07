use bevy::ecs::{
	schedule::{NextState, States},
	system::ResMut,
};
use common::traits::get_state::GetState;

pub fn set_state<TState: States + GetState<TOption>, TOption>(
	mut next_state: ResMut<NextState<TState>>,
) {
	next_state.set(TState::get_state());
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		prelude::State,
	};

	#[derive(Default, States, Debug, Hash, Eq, PartialEq, Clone)]
	enum _State {
		#[default]
		None,
		A,
	}

	struct _A;

	impl GetState<_A> for _State {
		fn get_state() -> Self {
			_State::A
		}
	}

	#[test]
	fn toggle_on() {
		let mut app = App::new();

		app.init_state::<_State>();
		app.add_systems(Update, set_state::<_State, _A>);
		app.update();
		app.update();

		let state = app.world.get_resource::<State<_State>>().unwrap();

		assert_eq!(&_State::A, state.get());
	}
}
