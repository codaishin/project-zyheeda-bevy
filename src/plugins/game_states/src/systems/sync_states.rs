use crate::{
	resources::game_state_context::GameStateContext,
	states::activity::Activity,
	system_params::ui_states::UIStates,
};
use bevy::prelude::*;
use common::prelude::*;
use std::collections::HashSet;

impl GameStateContext {
	pub(crate) fn sync_states(mut ctx: ResMut<Self>, activity: Res<State<Activity>>, ui: UIStates) {
		if !activity.is_changed() && !ui.is_changed() {
			return;
		}

		ctx.activity = ActivityState::from(activity.get());
		ctx.ui = HashSet::from_iter(UIState::iterator().filter(|ui_state| ui.is_on(ui_state)));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		states::{
			activity::Activity,
			ui::{ComboOverview, Hud, Inventory, Settings},
		},
		system_params::ui_states::UIStates,
	};
	use bevy::state::{app::StatesPlugin, state::FreelyMutableState};
	use test_case::test_case;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(StatesPlugin);
		UIStates::init(&mut app);
		app.init_state::<Activity>();
		app.init_resource::<GameStateContext>();
		app.add_systems(Update, GameStateContext::sync_states);

		app
	}

	#[test]
	fn sync_activity() {
		let mut app = setup();
		app.insert_state(Activity(ActivityState::Play));

		app.update();

		assert_eq!(
			(ActivityState::Play, &HashSet::default()),
			(
				app.world().resource::<GameStateContext>().activity,
				&app.world().resource::<GameStateContext>().ui,
			),
		);
	}

	#[test_case(Hud::On, UIState::Hud; "hud")]
	#[test_case(Inventory::On, UIState::Inventory; "inventory")]
	#[test_case(ComboOverview::On, UIState::ComboOverview; "combos")]
	#[test_case(Settings::On, UIState::Settings; "settings")]
	fn sync_ui<T>(state: T, ui: UIState)
	where
		T: FreelyMutableState,
	{
		let mut app = setup();
		app.insert_state(state);

		app.update();

		assert_eq!(
			&HashSet::from([ui]),
			&app.world().resource::<GameStateContext>().ui
		);
	}

	#[test]
	fn sync_uis() {
		let mut app = setup();
		app.insert_state(Hud::On);
		app.insert_state(ComboOverview::On);

		app.update();

		assert_eq!(
			&HashSet::from([UIState::Hud, UIState::ComboOverview]),
			&app.world().resource::<GameStateContext>().ui
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		app.insert_state(Activity(ActivityState::Play));
		app.insert_state(Inventory::On);

		app.update();
		app.world_mut().resource_mut::<GameStateContext>().activity = ActivityState::Save;
		app.world_mut().resource_mut::<GameStateContext>().ui = HashSet::from([UIState::Hud]);
		app.update();

		assert_eq!(
			(ActivityState::Save, &HashSet::from([UIState::Hud])),
			(
				app.world().resource::<GameStateContext>().activity,
				&app.world().resource::<GameStateContext>().ui
			)
		);
	}

	#[test]
	fn act_again_if_activity_changed() {
		let mut app = setup();
		app.insert_state(Activity(ActivityState::Play));

		app.update();
		app.insert_state(Activity(ActivityState::Paused));
		app.update();

		assert_eq!(
			ActivityState::Paused,
			app.world().resource::<GameStateContext>().activity,
		);
	}

	#[test_case(Hud::On, UIState::Hud; "hud")]
	#[test_case(Inventory::On, UIState::Inventory; "inventory")]
	#[test_case(ComboOverview::On, UIState::ComboOverview; "combos")]
	#[test_case(Settings::On, UIState::Settings; "settings")]
	fn act_again_if_ui_changed<T>(state: T, ui: UIState)
	where
		T: FreelyMutableState,
	{
		let mut app = setup();

		app.update();
		app.insert_state(state);
		app.update();

		assert_eq!(
			&HashSet::from([ui]),
			&app.world().resource::<GameStateContext>().ui,
		);
	}
}
