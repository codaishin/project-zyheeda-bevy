use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::prelude::*;
use std::collections::HashMap;

pub(crate) fn set_game_state<TGameStatesMut, TInput>(
	key_map: HashMap<ActionKey, GameState>,
) -> impl IntoSystem<(), (), ()>
where
	TGameStatesMut: ThreadSafe + for<'w, 's> SystemParam<Item<'w, 's>: GameStatesMut>,
	TInput: ThreadSafe + for<'w, 's> SystemParam<Item<'w, 's>: GetAllInputStates>,
{
	#[rustfmt::skip]
	let system = move |
		mut game_states: StaticSystemParam<TGameStatesMut>,
		input: StaticSystemParam<TInput>
	| {
		let triggers = input
			.get_all_input_states()
			.filter_map(just_pressed_action)
			.filter_map(|a| key_map.get(&a).copied());

		for trigger in triggers {
			let Some(setter) = game_states.get_game_state_setter(trigger) else {
				continue;
			};

			setter.set_game_state();
		}
	};

	IntoSystem::into_system(system)
}

fn just_pressed_action((a, i): (ActionKey, InputState)) -> Option<ActionKey> {
	match i {
		InputState::Pressed { just_now: true } => Some(a),
		_ => None,
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use bevy::app::{App, Update};
	use macros::NestedMocks;
	use mockall::automock;
	use std::collections::HashSet;
	use testing::{NestedMocks, SingleThreadedApp};
	use zyheeda_core::hash_map;

	#[derive(Resource, NestedMocks)]
	struct _Input {
		mock: Mock_Input,
	}

	#[automock]
	impl GetAllInputStates for _Input {
		fn get_all_input_states<TAction>(&self) -> impl Iterator<Item = (TAction, InputState)>
		where
			TAction: Into<ActionKey> + IterFinite + 'static,
		{
			self.mock.get_all_input_states()
		}
	}

	#[derive(Resource, Debug, PartialEq)]
	struct _GameStates {
		activity: GameState,
	}

	impl GameStatesMut for _GameStates {
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
			panic!("SHOULD NOT BE USED")
		}
	}

	struct _Setter<'a> {
		new: GameState,
		current: &'a mut GameState,
	}

	impl SetGameState for _Setter<'_> {
		fn set_game_state(self) {
			*self.current = self.new
		}
	}

	fn setup(input: _Input, game_states: _GameStates, keys: HashMap<ActionKey, GameState>) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(input);
		app.insert_resource(game_states);
		app.add_systems(
			Update,
			set_game_state::<ResMut<_GameStates>, Res<_Input>>(keys),
		);

		app
	}

	#[test]
	fn set() {
		let input = _Input::new().with_mock(|mock| {
			mock.expect_get_all_input_states().returning(|| {
				Box::new(std::iter::once((
					ActionKey::Slot(HandSlot::Left),
					InputState::just_pressed(),
				)))
			});
		});
		let game_states = _GameStates {
			activity: GameState::NewGame,
		};
		let keys = hash_map! {
			ActionKey::Slot(HandSlot::Left) => GameState::Play,
		};
		let mut app = setup(input, game_states, keys);

		app.update();

		assert_eq!(
			&_GameStates {
				activity: GameState::Play
			},
			app.world().resource::<_GameStates>()
		);
	}

	#[test]
	fn do_not_set_if_not_just_pressed() {
		let input = _Input::new().with_mock(|mock| {
			mock.expect_get_all_input_states().returning(|| {
				Box::new(std::iter::once((
					ActionKey::Slot(HandSlot::Left),
					InputState::just_released(),
				)))
			});
		});
		let game_states = _GameStates {
			activity: GameState::NewGame,
		};
		let keys = hash_map! {
			ActionKey::Slot(HandSlot::Left) => GameState::Play,
		};
		let mut app = setup(input, game_states, keys);

		app.update();

		assert_eq!(
			&_GameStates {
				activity: GameState::NewGame
			},
			app.world().resource::<_GameStates>()
		);
	}
}
