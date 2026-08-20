use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::prelude::*;
use std::collections::HashMap;

pub(crate) fn toggle_ui<TGameStatesMut, TInput>(
	keys: HashMap<ActionKey, IngameUI>,
) -> impl IntoSystem<(), (), ()>
where
	TGameStatesMut: ThreadSafe + for<'w, 's> SystemParam<Item<'w, 's>: GameStatesMut>,
	TInput: ThreadSafe + for<'w, 's> SystemParam<Item<'w, 's>: GetAllInputStates>,
{
	let system = move |mut game_states: StaticSystemParam<TGameStatesMut>,
	                   input: StaticSystemParam<TInput>| {
		let ui = game_states.ui_mut();
		let triggers = input
			.get_all_input_states()
			.filter_map(just_pressed_action)
			.filter_map(|a| keys.get(&a));

		for trigger in triggers {
			if ui.remove(trigger) {
				continue;
			}

			ui.insert(*trigger);
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
	use super::*;
	use macros::NestedMocks;
	use mockall::automock;
	use std::collections::HashSet;
	use testing::{NestedMocks, SingleThreadedApp};
	use zyheeda_core::prelude::*;

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

	#[derive(Resource, Debug, PartialEq, Default)]
	struct _GameStates {
		ui: HashSet<IngameUI>,
	}

	impl GameStatesMut for _GameStates {
		fn set_activity(&mut self, _: SettableActivity) {
			panic!("SHOULD NOT BE USED")
		}

		fn ui_mut(&mut self) -> &'_ mut HashSet<IngameUI> {
			&mut self.ui
		}
	}

	fn setup(input: _Input, game_states: _GameStates, keys: HashMap<ActionKey, IngameUI>) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(input);
		app.insert_resource(game_states);
		app.add_systems(Update, toggle_ui::<ResMut<_GameStates>, Res<_Input>>(keys));

		app
	}

	#[test]
	fn toggle_on() {
		let input = _Input::new().with_mock(|mock| {
			mock.expect_get_all_input_states().returning(|| {
				Box::new(std::iter::once((
					ActionKey::Slot(HandSlot::Left),
					InputState::just_pressed(),
				)))
			});
		});
		let game_states = _GameStates::default();
		let keys = hash_map! {
			ActionKey::Slot(HandSlot::Left) => IngameUI::Inventory,
		};
		let mut app = setup(input, game_states, keys);

		app.update();

		assert_eq!(
			&_GameStates {
				ui: HashSet::from([IngameUI::Inventory])
			},
			app.world().resource::<_GameStates>(),
		);
	}

	#[test]
	fn toggle_off() {
		let input = _Input::new().with_mock(|mock| {
			mock.expect_get_all_input_states().returning(|| {
				Box::new(std::iter::once((
					ActionKey::Slot(HandSlot::Left),
					InputState::just_pressed(),
				)))
			});
		});
		let game_states = _GameStates {
			ui: HashSet::from([IngameUI::Inventory]),
		};
		let keys = hash_map! {
			ActionKey::Slot(HandSlot::Left) => IngameUI::Inventory,
		};
		let mut app = setup(input, game_states, keys);

		app.update();

		assert_eq!(
			&_GameStates {
				ui: HashSet::from([])
			},
			app.world().resource::<_GameStates>(),
		);
	}

	#[test]
	fn do_not_toggle_if_not_just_pressed() {
		let input = _Input::new().with_mock(|mock| {
			mock.expect_get_all_input_states().returning(|| {
				Box::new(std::iter::once((
					ActionKey::Slot(HandSlot::Left),
					InputState::just_released(),
				)))
			});
		});
		let game_states = _GameStates::default();
		let keys = hash_map! {
			ActionKey::Slot(HandSlot::Left) => IngameUI::Inventory,
		};
		let mut app = setup(input, game_states, keys);

		app.update();

		assert_eq!(
			&_GameStates {
				ui: HashSet::from([])
			},
			app.world().resource::<_GameStates>(),
		);
	}
}
