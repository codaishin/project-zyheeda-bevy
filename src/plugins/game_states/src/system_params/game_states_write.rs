use crate::{
	resources::{activity_context::ActivityContext, ui_context::UIContext},
	states::activity::Activity,
	system_params::ui_states::UIStatesMut,
};
use bevy::{ecs::system::SystemParam, prelude::*};
use common::traits::{
	handles_game_states::{
		ActivityState,
		GameStateCollection,
		GameStateCollectionMut,
		GameStates,
		GameStatesMut,
		UIState,
	},
	iteration::IterFinite,
};
use std::collections::HashSet;

#[derive(SystemParam)]
pub struct GameStatesWrite<'w, 's> {
	activity_ctx: ResMut<'w, ActivityContext>,
	ui_ctx: ResMut<'w, UIContext>,
	activity: ResMut<'w, NextState<Activity>>,
	ui_states: UIStatesMut<'w>,
	cache: Local<'s, Option<(ActivityState, HashSet<UIState>)>>,
}

impl GameStates for GameStatesWrite<'_, '_> {
	fn game_states(&self) -> GameStateCollection<'_> {
		GameStateCollection {
			activity: self.activity_ctx.activity,
			ui: &self.ui_ctx.ui,
		}
	}
}

impl GameStatesMut for GameStatesWrite<'_, '_> {
	fn game_states_mut(&mut self) -> GameStateCollectionMut<'_> {
		if self.cache.is_none() {
			*self.cache = Some((self.activity_ctx.activity, self.ui_ctx.ui.clone()));
		}

		GameStateCollectionMut {
			activity: &mut self.activity_ctx.activity,
			ui: &mut self.ui_ctx.ui,
		}
	}
}

impl Drop for GameStatesWrite<'_, '_> {
	fn drop(&mut self) {
		let Some((ref old_activity, ref old_ui_states)) = self.cache.take() else {
			return;
		};

		if &self.activity_ctx.activity != old_activity {
			self.activity.set(Activity(self.activity_ctx.activity));
		}

		for state in UIState::iterator() {
			match Change::of(&state, old_ui_states, &self.ui_ctx.ui) {
				Some(Change::Added) => self.ui_states.set_on(state),
				Some(Change::Removed) => self.ui_states.set_off(&state),
				None => continue,
			}
		}
	}
}

enum Change {
	Added,
	Removed,
}

impl Change {
	fn of(state: &UIState, old: &HashSet<UIState>, current: &HashSet<UIState>) -> Option<Change> {
		match (old.contains(state), current.contains(state)) {
			(true, false) => Some(Change::Removed),
			(false, true) => Some(Change::Added),
			_ => None,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		states::ui::{ComboOverview, Hud, Inventory, Settings},
		system_params::ui_states::UIStates,
	};
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		state::{app::StatesPlugin, state::FreelyMutableState},
	};
	use common::traits::handles_game_states::{ActivityState, UIState};
	use std::marker::PhantomData;
	use test_case::test_case;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(StatesPlugin);
		app.init_state::<Activity>();
		UIStates::init(&mut app);
		app.init_resource::<ActivityContext>();
		app.init_resource::<UIContext>();

		app
	}

	macro_rules! assert_state_eq {
		($left:expr, $right:expr) => {{
			use NextState::*;

			match ($left, $right) {
				(Unchanged, Unchanged) => {}
				(Pending(left), Pending(right)) => assert_eq!(left, right),
				(PendingIfNeq(left), PendingIfNeq(right)) => {
					assert_eq!(left, right)
				}
				(left, right) => {
					panic!("assertion: `left == right` failed\n  left: {left:?}\n right: {right:?}")
				}
			}
		}};
	}

	#[test]
	fn set_activity() -> Result<(), RunSystemError> {
		let mut app = setup();

		app.world_mut()
			.run_system_once(move |mut w: GameStatesWrite| {
				*w.game_states_mut().activity = ActivityState::Paused;
			})?;

		assert_state_eq!(
			&NextState::Pending(Activity(ActivityState::Paused)),
			app.world().resource::<NextState<Activity>>()
		);
		Ok(())
	}

	#[test_case(UIState::Hud, Hud::On; "hud")]
	#[test_case(UIState::Inventory, Inventory::On; "inventory")]
	#[test_case(UIState::ComboOverview, ComboOverview::On; "combos")]
	#[test_case(UIState::Settings, Settings::On; "settings")]
	fn add_ui<TState>(state: UIState, expected: TState) -> Result<(), RunSystemError>
	where
		TState: FreelyMutableState,
	{
		let mut app = setup();

		app.world_mut()
			.run_system_once(move |mut w: GameStatesWrite| {
				w.game_states_mut().ui.insert(state);
			})?;

		assert_state_eq!(
			&NextState::Pending(expected),
			app.world().resource::<NextState<TState>>()
		);
		Ok(())
	}

	#[test_case(UIState::Hud, Hud::Off; "hud")]
	#[test_case(UIState::Inventory, Inventory::Off; "inventory")]
	#[test_case(UIState::ComboOverview, ComboOverview::Off; "combos")]
	#[test_case(UIState::Settings, Settings::Off; "settings")]
	fn remove_ui<TState>(state: UIState, expected: TState) -> Result<(), RunSystemError>
	where
		TState: FreelyMutableState,
	{
		let mut app = setup();
		app.world_mut()
			.run_system_once(move |mut w: GameStatesWrite| {
				w.game_states_mut().ui.insert(state);
			})?;

		app.world_mut()
			.run_system_once(move |mut w: GameStatesWrite| {
				w.game_states_mut().ui.remove(&state);
			})?;

		assert_state_eq!(
			&NextState::Pending(expected),
			app.world().resource::<NextState<TState>>()
		);
		Ok(())
	}

	#[test_case(PhantomData::<Activity>; "activity")]
	#[test_case(PhantomData::<Hud>; "hud")]
	#[test_case(PhantomData::<Inventory>; "inventory")]
	#[test_case(PhantomData::<ComboOverview>; "combos")]
	#[test_case(PhantomData::<Settings>; "settings")]
	fn do_nothing_if_not_changed<TState>(_: PhantomData<TState>) -> Result<(), RunSystemError>
	where
		TState: FreelyMutableState,
	{
		let mut app = setup();

		app.world_mut().run_system_once(|mut p: GameStatesWrite| {
			p.game_states_mut();
		})?;

		assert_state_eq!(
			&NextState::<TState>::Unchanged,
			app.world().resource::<NextState<TState>>()
		);
		Ok(())
	}

	#[test_case(|s| *s.activity = ActivityState::Play, |s| *s.activity = ActivityState::LoadingEssentialAssets, Activity(ActivityState::LoadingEssentialAssets); "activity")]
	#[test_case(|s| { s.ui.insert(UIState::Hud); }, |s| { s.ui.remove(&UIState::Hud); }, Hud::Off; "hud")]
	#[test_case(|s| { s.ui.insert(UIState::Inventory); }, |s| { s.ui.remove(&UIState::Inventory); }, Inventory::Off; "inventory")]
	#[test_case(|s| { s.ui.insert(UIState::ComboOverview); }, |s| { s.ui.remove(&UIState::ComboOverview); }, ComboOverview::Off; "combos")]
	#[test_case(|s| { s.ui.insert(UIState::Settings); }, |s| { s.ui.remove(&UIState::Settings); }, Settings::Off; "settings")]
	fn reevaluate_change_in_later_frame<TState>(
		op_a: fn(GameStateCollectionMut),
		op_b: fn(GameStateCollectionMut),
		expected: TState,
	) where
		TState: FreelyMutableState,
	{
		fn alternate(
			op_a: fn(GameStateCollectionMut),
			op_b: fn(GameStateCollectionMut),
		) -> impl Fn(GameStatesWrite, Local<bool>) {
			move |mut p, mut use_b| {
				if *use_b {
					op_b(p.game_states_mut());
				} else {
					op_a(p.game_states_mut());
				}

				*use_b = !*use_b;
			}
		}

		let mut app = setup();

		app.add_systems(Update, alternate(op_a, op_b));
		app.update();
		app.update();

		assert_state_eq!(
			&NextState::<TState>::Pending(expected),
			app.world().resource::<NextState<TState>>()
		);
	}
}
