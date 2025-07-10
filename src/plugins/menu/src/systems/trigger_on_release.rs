use crate::{
	components::button_interaction::ButtonInteraction,
	traits::{is_released::IsReleased, trigger_game_state::TriggerState},
};
use bevy::prelude::*;

impl<T> TriggerOnRelease for T where T: Component + TriggerState {}

pub(crate) trait TriggerOnRelease: Component + TriggerState + Sized {
	fn trigger_on_release(
		next_state: ResMut<NextState<Self::TState>>,
		triggers: Query<(&Self, &ButtonInteraction)>,
	) {
		trigger_on_release(next_state, triggers);
	}
}

fn trigger_on_release<TComponent, TInteraction>(
	mut next_state: ResMut<NextState<TComponent::TState>>,
	triggers: Query<(&TComponent, &TInteraction)>,
) where
	TComponent: Component + TriggerState,
	TInteraction: Component + IsReleased,
{
	for (trigger, interaction) in triggers {
		if !interaction.is_released() {
			continue;
		}
		next_state.set(trigger.trigger_state());
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

	impl TriggerState for _Component {
		type TState = _State;

		fn trigger_state(&self) -> Self::TState {
			_State::B
		}
	}

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
			.run_system_once(trigger_on_release::<_Component, _Released>)?;

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
	fn do_not_set_next_state_if_not_released() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn((_Component, _Released(false)));

		app.world_mut()
			.run_system_once(trigger_on_release::<_Component, _Released>)?;

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
