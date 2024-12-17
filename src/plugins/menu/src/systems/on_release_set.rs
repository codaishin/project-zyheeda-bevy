use crate::{components::button_interaction::ButtonInteraction, traits::is_released::IsReleased};
use bevy::{prelude::*, state::state::FreelyMutableState};

impl<T> OnReleaseSet for T where T: Component {}

pub(crate) trait OnReleaseSet
where
	Self: Component,
{
	fn on_release_set<TState>(
		state: TState,
	) -> impl Fn(ResMut<NextState<TState>>, Query<&ButtonInteraction, With<Self>>)
	where
		Self: Sized,
		TState: FreelyMutableState + Copy,
	{
		on_release_set(state)
	}
}

fn on_release_set<TState, TComponent, TInteraction>(
	state: TState,
) -> impl Fn(ResMut<NextState<TState>>, Query<&TInteraction, With<TComponent>>)
where
	TComponent: Component,
	TInteraction: Component + IsReleased,
	TState: FreelyMutableState + Copy,
{
	move |mut next_state: ResMut<NextState<TState>>,
	      interactions: Query<&TInteraction, With<TComponent>>| {
		let none_released = interactions
			.iter()
			.all(|interaction| !interaction.is_released());

		if none_released {
			return;
		};

		next_state.set(state);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		state::app::StatesPlugin,
	};

	#[derive(States, Default, Debug, PartialEq, Eq, Hash, Clone, Copy)]
	enum _State {
		#[default]
		A,
		B,
	}

	#[derive(Component)]
	struct _Component;

	#[derive(Component)]
	struct _Released(bool);

	impl IsReleased for _Released {
		fn is_released(&self) -> bool {
			self.0
		}
	}

	fn setup() -> App {
		let mut app = App::new();
		app.add_plugins(StatesPlugin);
		app.init_state::<_State>();

		app
	}

	#[test]
	fn set_state_when_released() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn((_Component, _Released(true)));

		app.world_mut()
			.run_system_once(on_release_set::<_State, _Component, _Released>(_State::B))?;

		let state = app.world().resource::<NextState<_State>>();
		assert!(
			matches!(state, NextState::Pending(_State::B)),
			"expected: {:?}\n     got: {:?}",
			NextState::Pending(_State::B),
			state,
		);
		Ok(())
	}

	#[test]
	fn do_set_next_state_if_not_released() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn((_Component, _Released(false)));

		app.world_mut()
			.run_system_once(on_release_set::<_State, _Component, _Released>(_State::B))?;

		let state = app.world().resource::<NextState<_State>>();
		assert!(
			matches!(state, NextState::Unchanged),
			"expected: {:?}\n     got: {:?}",
			NextState::<_State>::Unchanged,
			state,
		);
		Ok(())
	}

	#[test]
	fn do_set_next_state_if_component_not_present() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn(_Released(true));

		app.world_mut()
			.run_system_once(on_release_set::<_State, _Component, _Released>(_State::B))?;

		let state = app.world().resource::<NextState<_State>>();
		assert!(
			matches!(state, NextState::Unchanged),
			"expected: {:?}\n     got: {:?}",
			NextState::<_State>::Unchanged,
			state,
		);
		Ok(())
	}
}
