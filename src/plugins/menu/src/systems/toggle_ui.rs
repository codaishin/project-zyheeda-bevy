use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::{prelude::*, states::menu_state::MenuState};
use std::collections::HashMap;

pub(crate) fn toggle_ui<TGameStatesMut, TInput>(
	keys: HashMap<MenuState, IngameUI>,
) -> impl IntoSystem<(), (), ()>
where
	TGameStatesMut: ThreadSafe + for<'w, 's> SystemParam<Item<'w, 's>: GameStatesMut>,
	TInput: ThreadSafe + for<'w, 's> SystemParam<Item<'w, 's>: GetAllInputStates>,
{
	let system = move |mut game_states: StaticSystemParam<TGameStatesMut>,
	                   input: StaticSystemParam<TInput>| {
		let ui = game_states.ui_mut();
		let ingame_ui = input
			.get_all_input_states()
			.filter_map(just_pressed_menu)
			.find_map(|a| keys.get(&a));

		match ingame_ui {
			Some(ingame_ui) if ui.contains(ingame_ui) => {
				ui.clear();
				ui.insert(IngameUI::Hud);
			}
			Some(ingame_ui) => {
				ui.clear();
				ui.insert(*ingame_ui);
			}
			None if ui.is_empty() => {
				ui.insert(IngameUI::Hud);
			}
			None => {}
		}
	};

	IntoSystem::into_system(system)
}

fn just_pressed_menu((m, i): (MenuState, InputState)) -> Option<MenuState> {
	match i {
		InputState::Pressed { just_now: true } => Some(m),
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

	fn setup(input: _Input, game_states: _GameStates, keys: HashMap<MenuState, IngameUI>) -> App {
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
					MenuState::Inventory,
					InputState::just_pressed(),
				)))
			});
		});
		let game_states = _GameStates::default();
		let keys = hash_map! {
			MenuState::Inventory => IngameUI::Inventory,
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
					MenuState::Inventory,
					InputState::just_pressed(),
				)))
			});
		});
		let game_states = _GameStates {
			ui: HashSet::from([IngameUI::Inventory]),
		};
		let keys = hash_map! {
			MenuState::Inventory => IngameUI::Inventory,
		};
		let mut app = setup(input, game_states, keys);

		app.update();

		assert_eq!(
			&_GameStates {
				ui: HashSet::from([IngameUI::Hud])
			},
			app.world().resource::<_GameStates>(),
		);
	}

	#[test]
	fn toggle_on_and_toggle_off_all_others() {
		let input = _Input::new().with_mock(|mock| {
			mock.expect_get_all_input_states().returning(|| {
				Box::new(std::iter::once((
					MenuState::Inventory,
					InputState::just_pressed(),
				)))
			});
		});
		let game_states = _GameStates {
			ui: HashSet::from([IngameUI::Settings, IngameUI::ComboOverview, IngameUI::Hud]),
		};
		let keys = hash_map! {
			MenuState::Inventory => IngameUI::Inventory,
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
	fn do_nothing_if_not_just_pressed() {
		let input = _Input::new().with_mock(|mock| {
			mock.expect_get_all_input_states().returning(|| {
				Box::new(std::iter::once((
					MenuState::Inventory,
					InputState::just_released(),
				)))
			});
		});
		let game_states = _GameStates {
			ui: HashSet::from([IngameUI::Inventory]),
		};
		let keys = hash_map! {
			MenuState::Inventory => IngameUI::Inventory,
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
	fn toggle_on_hud_if_not_just_pressed_but_no_ui_is_on() {
		let input = _Input::new().with_mock(|mock| {
			mock.expect_get_all_input_states().returning(|| {
				Box::new(std::iter::once((
					MenuState::Inventory,
					InputState::just_released(),
				)))
			});
		});
		let game_states = _GameStates {
			ui: HashSet::from([]),
		};
		let keys = hash_map! {
			MenuState::Inventory => IngameUI::Inventory,
		};
		let mut app = setup(input, game_states, keys);

		app.update();

		assert_eq!(
			&_GameStates {
				ui: HashSet::from([IngameUI::Hud])
			},
			app.world().resource::<_GameStates>(),
		);
	}
}
