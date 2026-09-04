use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::prelude::*;
use std::collections::HashMap;

pub(crate) fn toggle_ui<TGameStatesMut, TInput>(
	keys: HashMap<MenuKey, Gui>,
) -> impl IntoSystem<(), (), ()>
where
	TGameStatesMut: ThreadSafe + for<'w, 's> SystemParam<Item<'w, 's>: GameStatesMut>,
	TInput: ThreadSafe + for<'w, 's> SystemParam<Item<'w, 's>: GetAllInputStates>,
{
	let system = move |mut game_states: StaticSystemParam<TGameStatesMut>,
	                   input: StaticSystemParam<TInput>| {
		let gui = game_states.gui_mut();
		let ingame_ui = input
			.get_all_input_states()
			.filter_map(just_pressed_menu)
			.find_map(|a| keys.get(&a));

		match ingame_ui {
			Some(ingame_ui) if gui.contains(ingame_ui) => {
				gui.clear();
				gui.insert(Gui::Hud);
			}
			Some(ingame_ui) => {
				gui.clear();
				gui.insert(*ingame_ui);
			}
			None if gui.is_empty() => {
				gui.insert(Gui::Hud);
			}
			None => {}
		}
	};

	IntoSystem::into_system(system)
}

fn just_pressed_menu((m, i): (MenuKey, InputState)) -> Option<MenuKey> {
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
		ui: HashSet<Gui>,
	}

	impl GameStatesMut for _GameStates {
		type TGameStateSetter<'a>
			= _Setter
		where
			Self: 'a;

		fn get_game_state_setter(&mut self, _: GameState) -> Option<_Setter> {
			panic!("SHOULD NOT BE USED");
		}

		fn gui_mut(&mut self) -> &'_ mut HashSet<Gui> {
			&mut self.ui
		}
	}

	struct _Setter;

	impl SetGameState for _Setter {
		fn set_game_state(self) {}
	}

	fn setup(input: _Input, game_states: _GameStates, keys: HashMap<MenuKey, Gui>) -> App {
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
					MenuKey::Inventory,
					InputState::just_pressed(),
				)))
			});
		});
		let game_states = _GameStates::default();
		let keys = hash_map! {
			MenuKey::Inventory => Gui::Inventory,
		};
		let mut app = setup(input, game_states, keys);

		app.update();

		assert_eq!(
			&_GameStates {
				ui: HashSet::from([Gui::Inventory])
			},
			app.world().resource::<_GameStates>(),
		);
	}

	#[test]
	fn toggle_off() {
		let input = _Input::new().with_mock(|mock| {
			mock.expect_get_all_input_states().returning(|| {
				Box::new(std::iter::once((
					MenuKey::Inventory,
					InputState::just_pressed(),
				)))
			});
		});
		let game_states = _GameStates {
			ui: HashSet::from([Gui::Inventory]),
		};
		let keys = hash_map! {
			MenuKey::Inventory => Gui::Inventory,
		};
		let mut app = setup(input, game_states, keys);

		app.update();

		assert_eq!(
			&_GameStates {
				ui: HashSet::from([Gui::Hud])
			},
			app.world().resource::<_GameStates>(),
		);
	}

	#[test]
	fn toggle_on_and_toggle_off_all_others() {
		let input = _Input::new().with_mock(|mock| {
			mock.expect_get_all_input_states().returning(|| {
				Box::new(std::iter::once((
					MenuKey::Inventory,
					InputState::just_pressed(),
				)))
			});
		});
		let game_states = _GameStates {
			ui: HashSet::from([Gui::Settings, Gui::ComboOverview, Gui::Hud]),
		};
		let keys = hash_map! {
			MenuKey::Inventory => Gui::Inventory,
		};
		let mut app = setup(input, game_states, keys);

		app.update();

		assert_eq!(
			&_GameStates {
				ui: HashSet::from([Gui::Inventory])
			},
			app.world().resource::<_GameStates>(),
		);
	}

	#[test]
	fn do_nothing_if_not_just_pressed() {
		let input = _Input::new().with_mock(|mock| {
			mock.expect_get_all_input_states().returning(|| {
				Box::new(std::iter::once((
					MenuKey::Inventory,
					InputState::just_released(),
				)))
			});
		});
		let game_states = _GameStates {
			ui: HashSet::from([Gui::Inventory]),
		};
		let keys = hash_map! {
			MenuKey::Inventory => Gui::Inventory,
		};
		let mut app = setup(input, game_states, keys);

		app.update();

		assert_eq!(
			&_GameStates {
				ui: HashSet::from([Gui::Inventory])
			},
			app.world().resource::<_GameStates>(),
		);
	}

	#[test]
	fn toggle_on_hud_if_not_just_pressed_but_no_ui_is_on() {
		let input = _Input::new().with_mock(|mock| {
			mock.expect_get_all_input_states().returning(|| {
				Box::new(std::iter::once((
					MenuKey::Inventory,
					InputState::just_released(),
				)))
			});
		});
		let game_states = _GameStates {
			ui: HashSet::from([]),
		};
		let keys = hash_map! {
			MenuKey::Inventory => Gui::Inventory,
		};
		let mut app = setup(input, game_states, keys);

		app.update();

		assert_eq!(
			&_GameStates {
				ui: HashSet::from([Gui::Hud])
			},
			app.world().resource::<_GameStates>(),
		);
	}
}
