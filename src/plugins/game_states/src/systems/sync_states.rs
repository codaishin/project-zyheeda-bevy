use crate::{
	resources::game_state_context::GameStateContext,
	states::state_internal::StateInternal,
	system_params::gui_states::GuiStates,
};
use bevy::prelude::*;
use common::prelude::*;
use std::collections::HashSet;

impl GameStateContext {
	pub(crate) fn sync_states(
		mut ctx: ResMut<Self>,
		game_state: Res<State<StateInternal<GameState>>>,
		gui: GuiStates,
	) {
		if !game_state.is_changed() && !gui.is_changed() {
			return;
		}

		ctx.game_state = *game_state.get();
		ctx.gui = HashSet::from_iter(Gui::iterator().filter(|ui_state| gui.is_on(ui_state)));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		states::{
			gui::{ComboOverview, Hud, Inventory, Settings},
			state_internal::StateInternal,
		},
		system_params::gui_states::GuiStates,
	};
	use bevy::state::{app::StatesPlugin, state::FreelyMutableState};
	use test_case::test_case;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(StatesPlugin);
		GuiStates::init(&mut app);
		app.init_state::<StateInternal<GameState>>();
		app.init_resource::<GameStateContext>();
		app.add_systems(Update, GameStateContext::sync_states);

		app
	}

	#[test]
	fn sync_game_state() {
		let mut app = setup();
		app.insert_state(StateInternal::active(GameState::Play));

		app.update();

		assert_eq!(
			(StateInternal::active(GameState::Play), &HashSet::default()),
			(
				app.world().resource::<GameStateContext>().game_state,
				&app.world().resource::<GameStateContext>().gui,
			),
		);
	}

	#[test_case(Hud::On, Gui::Hud; "hud")]
	#[test_case(Inventory::On, Gui::Inventory; "inventory")]
	#[test_case(ComboOverview::On, Gui::ComboOverview; "combos")]
	#[test_case(Settings::On, Gui::Settings; "settings")]
	fn sync_gui<T>(state: T, ui: Gui)
	where
		T: FreelyMutableState,
	{
		let mut app = setup();
		app.insert_state(state);

		app.update();

		assert_eq!(
			&HashSet::from([ui]),
			&app.world().resource::<GameStateContext>().gui
		);
	}

	#[test]
	fn sync_guis() {
		let mut app = setup();
		app.insert_state(Hud::On);
		app.insert_state(ComboOverview::On);

		app.update();

		assert_eq!(
			&HashSet::from([Gui::Hud, Gui::ComboOverview]),
			&app.world().resource::<GameStateContext>().gui
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		app.insert_state(StateInternal::active(GameState::Play));
		app.insert_state(Inventory::On);

		app.update();
		app.world_mut()
			.resource_mut::<GameStateContext>()
			.game_state = StateInternal::active(GameState::Save);
		app.world_mut().resource_mut::<GameStateContext>().gui = HashSet::from([Gui::Hud]);
		app.update();

		assert_eq!(
			(
				StateInternal::active(GameState::Save),
				&HashSet::from([Gui::Hud])
			),
			(
				app.world().resource::<GameStateContext>().game_state,
				&app.world().resource::<GameStateContext>().gui
			)
		);
	}

	#[test]
	fn act_again_if_game_state_changed() {
		let mut app = setup();
		app.insert_state(StateInternal::active(GameState::Play));

		app.update();
		app.insert_state(StateInternal::active(GameState::Pause));
		app.update();

		assert_eq!(
			StateInternal::active(GameState::Pause),
			app.world().resource::<GameStateContext>().game_state,
		);
	}

	#[test_case(Hud::On, Gui::Hud; "hud")]
	#[test_case(Inventory::On, Gui::Inventory; "inventory")]
	#[test_case(ComboOverview::On, Gui::ComboOverview; "combos")]
	#[test_case(Settings::On, Gui::Settings; "settings")]
	fn act_again_if_gui_changed<T>(state: T, ui: Gui)
	where
		T: FreelyMutableState,
	{
		let mut app = setup();

		app.update();
		app.insert_state(state);
		app.update();

		assert_eq!(
			&HashSet::from([ui]),
			&app.world().resource::<GameStateContext>().gui,
		);
	}
}
