use crate::{
	components::button_interaction::ButtonInteraction,
	traits::{is_released::IsReleased, trigger_game_state::TriggerState},
};
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::prelude::*;

impl<T> TriggerOnRelease for T where T: Component + TriggerState {}

pub(crate) trait TriggerOnRelease: Component + TriggerState + Sized {
	fn trigger_on_release<TGameStatesMut>(
		states: StaticSystemParam<TGameStatesMut>,
		triggers: Query<(&Self, &ButtonInteraction)>,
	) where
		TGameStatesMut: for<'w, 's> SystemParam<Item<'w, 's>: GameStatesMut>,
	{
		trigger_on_release(states, triggers);
	}
}

fn trigger_on_release<TGameStatesMut, TComponent, TInteraction>(
	mut states: StaticSystemParam<TGameStatesMut>,
	triggers: Query<(&TComponent, &TInteraction)>,
) where
	TGameStatesMut: for<'w, 's> SystemParam<Item<'w, 's>: GameStatesMut>,
	TComponent: Component + TriggerState,
	TInteraction: Component + IsReleased,
{
	for (trigger, interaction) in triggers {
		if !interaction.is_released() {
			continue;
		}

		let Some(setter) = states.get_game_state_setter(trigger.trigger_state()) else {
			continue;
		};

		setter.set_game_state();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use std::collections::HashSet;

	#[derive(Component)]
	struct _Component {
		trigger: GameStateCommand,
	}

	impl TriggerState for _Component {
		fn trigger_state(&self) -> GameStateCommand {
			self.trigger
		}
	}

	#[derive(Component)]
	struct _Released(bool);

	impl IsReleased for _Released {
		fn is_released(&self) -> bool {
			self.0
		}
	}

	#[derive(Resource, Debug, PartialEq)]
	struct _States {
		activity: GameStateCommand,
		ui: HashSet<IngameUI>,
	}

	impl _States {
		fn from_activity(activity: GameStateCommand) -> Self {
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

		fn get_game_state_setter(&mut self, activity: GameStateCommand) -> Option<_Setter<'_>> {
			Some(_Setter {
				new: activity,
				current: &mut self.activity,
			})
		}

		fn ui_mut(&mut self) -> &'_ mut HashSet<IngameUI> {
			&mut self.ui
		}
	}

	struct _Setter<'a> {
		new: GameStateCommand,
		current: &'a mut GameStateCommand,
	}

	impl SetGameState for _Setter<'_> {
		fn set_game_state(self) {
			*self.current = self.new;
		}
	}

	fn setup(current: GameStateCommand) -> App {
		let mut app = App::new();

		app.insert_resource(_States::from_activity(current));

		app
	}

	#[test]
	fn trigger_when_released() -> Result<(), RunSystemError> {
		let mut app = setup(GameStateCommand::Pause);
		app.world_mut().spawn((
			_Component {
				trigger: GameStateCommand::Play,
			},
			_Released(true),
		));

		app.world_mut()
			.run_system_once(trigger_on_release::<ResMut<_States>, _Component, _Released>)?;

		assert_eq!(
			&_States::from_activity(GameStateCommand::Play),
			app.world().resource::<_States>()
		);
		Ok(())
	}

	#[test]
	fn do_not_trigger_when_not_released() -> Result<(), RunSystemError> {
		let mut app = setup(GameStateCommand::Pause);
		app.world_mut().spawn((
			_Component {
				trigger: GameStateCommand::Play,
			},
			_Released(false),
		));

		app.world_mut()
			.run_system_once(trigger_on_release::<ResMut<_States>, _Component, _Released>)?;

		assert_eq!(
			&_States::from_activity(GameStateCommand::Pause),
			app.world().resource::<_States>()
		);
		Ok(())
	}
}
