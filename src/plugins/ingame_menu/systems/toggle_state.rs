use crate::plugins::ingame_menu::traits::{inverted::Inverted, key_just_pressed::KeyJustPressed};
use bevy::{
	ecs::{
		component::Component,
		schedule::{NextState, States},
		system::ResMut,
	},
	prelude::{Input, KeyCode, Res, State},
};

pub fn toggle_state<
	TState: Inverted<TComponent> + States,
	TComponent: KeyJustPressed + Component,
>(
	keys: Res<Input<KeyCode>>,
	current_state: Res<State<TState>>,
	mut state: ResMut<NextState<TState>>,
) {
	if !TComponent::just_pressed(&keys) {
		return;
	}

	state.set(current_state.get().inverted());
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		ecs::schedule::States,
		prelude::State,
	};

	#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
	enum _State {
		#[default]
		Off,
		On,
	}

	#[derive(Component)]
	struct _Component;

	impl KeyJustPressed for _Component {
		fn just_pressed(input: &Input<KeyCode>) -> bool {
			input.pressed(KeyCode::Apps)
		}
	}

	impl<T> Inverted<T> for _State {
		fn inverted(&self) -> Self {
			match self {
				_State::On => _State::Off,
				_State::Off => _State::On,
			}
		}
	}

	fn setup() -> App {
		let mut app = App::new();

		app.insert_resource(Input::<KeyCode>::default());
		app.add_state::<_State>();
		app.add_systems(Update, toggle_state::<_State, _Component>);

		app
	}

	#[test]
	fn toggle_on() {
		let mut app = setup();
		let mut input = app.world.get_resource_mut::<Input<KeyCode>>().unwrap();
		input.press(KeyCode::Apps);

		app.update();
		app.update();

		let state = app.world.get_resource::<State<_State>>().unwrap();
		assert_eq!(&_State::On, state.get());
	}

	#[test]
	fn toggle_off() {
		let mut app = setup();
		let mut input = app.world.get_resource_mut::<Input<KeyCode>>().unwrap();
		input.press(KeyCode::Apps);

		app.update();
		app.update();
		app.update();

		let state = app.world.get_resource::<State<_State>>().unwrap();
		assert_eq!(&_State::Off, state.get());
	}

	#[test]
	fn no_toggle() {
		let mut app = setup();

		app.update();
		app.update();

		let state = app.world.get_resource::<State<_State>>().unwrap();
		assert_eq!(&_State::Off, state.get());
	}
}
