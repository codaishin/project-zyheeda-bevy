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

		states.set_activity(trigger.trigger_state());
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::traits::handles_game_states::SettableActivity;
	use std::collections::HashSet;

	#[derive(Component)]
	struct _Component {
		trigger: SettableActivity,
	}

	impl TriggerState for _Component {
		fn trigger_state(&self) -> SettableActivity {
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
		activity: SettableActivity,
		ui: HashSet<IngameUI>,
	}

	impl _States {
		fn from_activity(activity: SettableActivity) -> Self {
			Self {
				activity,
				ui: HashSet::default(),
			}
		}
	}

	impl GameStatesMut for _States {
		fn set_activity(&mut self, activity: SettableActivity) {
			self.activity = activity
		}

		fn ui_mut(&mut self) -> &'_ mut HashSet<IngameUI> {
			&mut self.ui
		}
	}

	fn setup(current: SettableActivity) -> App {
		let mut app = App::new();

		app.insert_resource(_States::from_activity(current));

		app
	}

	#[test]
	fn trigger_when_released() -> Result<(), RunSystemError> {
		let mut app = setup(SettableActivity::Paused);
		app.world_mut().spawn((
			_Component {
				trigger: SettableActivity::Play,
			},
			_Released(true),
		));

		app.world_mut()
			.run_system_once(trigger_on_release::<ResMut<_States>, _Component, _Released>)?;

		assert_eq!(
			&_States::from_activity(SettableActivity::Play),
			app.world().resource::<_States>()
		);
		Ok(())
	}

	#[test]
	fn do_not_trigger_when_not_released() -> Result<(), RunSystemError> {
		let mut app = setup(SettableActivity::Paused);
		app.world_mut().spawn((
			_Component {
				trigger: SettableActivity::Play,
			},
			_Released(false),
		));

		app.world_mut()
			.run_system_once(trigger_on_release::<ResMut<_States>, _Component, _Released>)?;

		assert_eq!(
			&_States::from_activity(SettableActivity::Paused),
			app.world().resource::<_States>()
		);
		Ok(())
	}
}
