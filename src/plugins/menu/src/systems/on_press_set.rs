use bevy::{prelude::*, state::state::FreelyMutableState};

impl<T> OnPressSet for T where T: Component {}

pub(crate) trait OnPressSet
where
	Self: Component,
{
	fn on_press_set<TState>(
		state: TState,
	) -> impl Fn(ResMut<NextState<TState>>, Query<&Interaction, With<Self>>)
	where
		Self: Sized,
		TState: FreelyMutableState + Copy,
	{
		move |mut next_state: ResMut<NextState<TState>>,
		      triggers: Query<&Interaction, With<Self>>| {
			let none_pressed = triggers
				.iter()
				.all(|interaction| interaction != &Interaction::Pressed);

			if none_pressed {
				return;
			};

			next_state.set(state);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{ecs::system::RunSystemOnce, state::app::StatesPlugin};

	#[derive(States, Default, Debug, PartialEq, Eq, Hash, Clone, Copy)]
	enum _State {
		#[default]
		A,
		B,
	}

	#[derive(Component)]
	struct _Component;

	fn setup() -> App {
		let mut app = App::new();
		app.add_plugins(StatesPlugin);
		app.init_state::<_State>();

		app
	}

	#[test]
	fn set_state_when_pressed() {
		let mut app = setup();
		app.world_mut().spawn((_Component, Interaction::Pressed));

		app.world_mut()
			.run_system_once(_Component::on_press_set(_State::B));

		let state = app.world().resource::<NextState<_State>>();
		assert!(
			matches!(state, NextState::Pending(_State::B)),
			"expected: {:?}\n     got: {:?}",
			NextState::Pending(_State::B),
			state,
		);
	}

	#[test]
	fn do_set_next_state_if_not_pressed() {
		let mut app = setup();
		app.world_mut().spawn((_Component, Interaction::None));
		app.world_mut().spawn((_Component, Interaction::Hovered));

		app.world_mut()
			.run_system_once(_Component::on_press_set(_State::B));

		let state = app.world().resource::<NextState<_State>>();
		assert!(
			matches!(state, NextState::Unchanged),
			"expected: {:?}\n     got: {:?}",
			NextState::<_State>::Unchanged,
			state,
		);
	}

	#[test]
	fn do_set_next_state_if_component_not_present() {
		let mut app = setup();
		app.world_mut().spawn(Interaction::Pressed);

		app.world_mut()
			.run_system_once(_Component::on_press_set(_State::B));

		let state = app.world().resource::<NextState<_State>>();
		assert!(
			matches!(state, NextState::Unchanged),
			"expected: {:?}\n     got: {:?}",
			NextState::<_State>::Unchanged,
			state,
		);
	}
}
