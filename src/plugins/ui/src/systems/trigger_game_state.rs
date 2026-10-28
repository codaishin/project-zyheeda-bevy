use crate::traits::trigger_game_state::TriggerState;
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
	ui_widgets::Activate,
};
use common::prelude::*;

impl<T> TriggerGameState for T where T: Component + TriggerState {}

pub(crate) trait TriggerGameState: Component + TriggerState + Sized {
	fn trigger_game_state<TGameStatesMut>(
		activated: On<Activate>,
		mut states: StaticSystemParam<TGameStatesMut>,
		buttons: Query<&Self>,
	) where
		TGameStatesMut: for<'w, 's> SystemParam<Item<'w, 's>: GameStatesMut>,
	{
		let Ok(button) = buttons.get(activated.entity) else {
			return;
		};

		let Some(setter) = states.get_game_state_setter(button.trigger_state()) else {
			return;
		};

		setter.set_game_state();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::RunSystemError;
	use std::collections::HashSet;

	#[derive(Component)]
	struct _Component {
		trigger: GameState,
	}

	impl TriggerState for _Component {
		fn trigger_state(&self) -> GameState {
			self.trigger
		}
	}

	#[derive(Resource, Debug, PartialEq)]
	struct _States {
		activity: GameState,
		ui: HashSet<Gui>,
	}

	impl _States {
		fn from_activity(activity: GameState) -> Self {
			Self {
				activity,
				ui: HashSet::default(),
			}
		}
	}

	impl GameStatesMut for _States {
		type TGameStateSetter<'a>
			= _Setter<'a>
		where
			Self: 'a;

		fn get_game_state_setter(&mut self, activity: GameState) -> Option<_Setter<'_>> {
			Some(_Setter {
				new: activity,
				current: &mut self.activity,
			})
		}

		fn gui_mut(&mut self) -> &'_ mut HashSet<Gui> {
			&mut self.ui
		}
	}

	struct _Setter<'a> {
		new: GameState,
		current: &'a mut GameState,
	}

	impl SetGameState for _Setter<'_> {
		fn set_game_state(self) {
			*self.current = self.new;
		}
	}

	fn setup(current: GameState) -> App {
		let mut app = App::new();

		app.insert_resource(_States::from_activity(current));
		app.add_observer(_Component::trigger_game_state::<ResMut<_States>>);

		app
	}

	#[test]
	fn trigger_when_released() -> Result<(), RunSystemError> {
		let mut app = setup(GameState::Pause);
		let mut entity = app.world_mut().spawn(_Component {
			trigger: GameState::Play,
		});

		entity.trigger(|entity| Activate { entity });

		assert_eq!(
			&_States::from_activity(GameState::Play),
			app.world().resource::<_States>()
		);
		Ok(())
	}
}
