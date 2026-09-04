use crate::{
	events::StateEvent,
	resources::game_state_context::GameStateContext,
	system_params::gui_states::GuiStatesMut,
};
use bevy::{ecs::system::SystemParam, prelude::*};
use common::prelude::{SetGameState as SetGameStateTrait, *};
use std::collections::HashSet;

#[derive(SystemParam)]
pub struct GameStatesWriteParam<'w, 's> {
	current: Res<'w, GameStateContext>,
	activity: ActivityStateMut<'w, 's>,
	ui: GuiStatesMut<'w>,
	game_state_change: Local<'s, Option<GameState>>,
	ui_change: Local<'s, Option<HashSet<Gui>>>,
}

impl GameStatesWriteParam<'_, '_> {
	fn drain_game_state_change(&mut self) {
		let Some(change) = self.game_state_change.take() else {
			return;
		};

		self.activity.set(change);
	}

	fn drain_gui_change(&mut self) {
		let Some(change) = self.ui_change.take() else {
			return;
		};

		let detect_change = |state| Change::of(state, &self.current.gui, &change);
		let changes = Gui::iterator().filter_map(detect_change);

		for change in changes {
			match change {
				Change::Added(state) => self.ui.set_on(state),
				Change::Removed(state) => self.ui.set_off(&state),
			}
		}
	}
}

impl GameStatesMut for GameStatesWriteParam<'_, '_> {
	type TGameStateSetter<'a>
		= SetGameState<'a>
	where
		Self: 'a;

	fn get_game_state_setter(&mut self, new: GameState) -> Option<SetGameState<'_>> {
		let command = match (self.current.game_state.try_into_active(), new) {
			(Some(current), new) if current != new => new,
			(Some(GameState::Pause), GameState::Pause) => GameState::Play,
			_ => return None,
		};

		Some(SetGameState {
			command,
			change: &mut self.game_state_change,
		})
	}

	fn gui_mut(&mut self) -> &'_ mut HashSet<Gui> {
		self.ui_change.get_or_insert(self.current.gui.clone())
	}
}

impl Drop for GameStatesWriteParam<'_, '_> {
	fn drop(&mut self) {
		self.drain_game_state_change();
		self.drain_gui_change();
	}
}

pub struct SetGameState<'a> {
	command: GameState,
	change: &'a mut Option<GameState>,
}

impl SetGameStateTrait for SetGameState<'_> {
	fn set_game_state(self) {
		*self.change = Some(self.command);
	}
}

#[derive(SystemParam)]
struct ActivityStateMut<'w, 's> {
	commands: ZyheedaCommands<'w, 's>,
}

impl ActivityStateMut<'_, '_> {
	fn set(&mut self, state: impl Into<StateEvent<GameState>>) {
		self.commands.trigger_observers_for(state.into());
	}
}

enum Change {
	Added(Gui),
	Removed(Gui),
}

impl Change {
	fn of(state: Gui, current: &HashSet<Gui>, queued: &HashSet<Gui>) -> Option<Change> {
		match (current.contains(&state), queued.contains(&state)) {
			(true, false) => Some(Change::Removed(state)),
			(false, true) => Some(Change::Added(state)),
			_ => None,
		}
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
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		state::{app::StatesPlugin, state::FreelyMutableState},
	};
	use std::marker::PhantomData;
	use test_case::test_case;
	use testing::SingleThreadedApp;

	#[derive(Resource, Debug, PartialEq)]
	struct _State(Option<StateEvent<GameState>>);

	impl _State {
		fn record(on_state: On<StateEvent<GameState>>, mut c: Commands) {
			c.insert_resource(_State(Some(*on_state)));
		}

		fn clear(mut state: ResMut<_State>) {
			state.0 = None;
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(StatesPlugin);
		app.add_observer(_State::record);
		app.add_systems(First, _State::clear);
		GuiStates::init(&mut app);
		app.insert_resource(_State(None));
		app.init_state::<StateInternal<GameState>>();
		app.init_resource::<GameStateContext>();

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
	fn set_game_state() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut()
			.resource_mut::<GameStateContext>()
			.game_state = StateInternal::active(GameState::NewGame);

		app.world_mut()
			.run_system_once(move |mut w: GameStatesWriteParam| {
				if let Some(s) = w.get_game_state_setter(GameState::Play) {
					s.set_game_state()
				};
			})?;

		assert_eq!(
			&_State(Some(StateEvent::Active(GameState::Play))),
			app.world().resource::<_State>()
		);
		Ok(())
	}

	#[test]
	fn set_game_state_paused_to_play() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut()
			.resource_mut::<GameStateContext>()
			.game_state = StateInternal::active(GameState::Pause);

		app.world_mut()
			.run_system_once(move |mut w: GameStatesWriteParam| {
				if let Some(s) = w.get_game_state_setter(GameState::Pause) {
					s.set_game_state()
				};
			})?;

		assert_eq!(
			&_State(Some(StateEvent::Active(GameState::Play))),
			app.world().resource::<_State>()
		);
		Ok(())
	}

	#[test]
	fn set_game_state_play_to_paused() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut()
			.resource_mut::<GameStateContext>()
			.game_state = StateInternal::active(GameState::Play);

		app.world_mut()
			.run_system_once(move |mut w: GameStatesWriteParam| {
				if let Some(s) = w.get_game_state_setter(GameState::Pause) {
					s.set_game_state()
				};
			})?;

		assert_eq!(
			&_State(Some(StateEvent::Active(GameState::Pause))),
			app.world().resource::<_State>()
		);
		Ok(())
	}

	#[test_case(Gui::Hud, Hud::On; "hud")]
	#[test_case(Gui::Inventory, Inventory::On; "inventory")]
	#[test_case(Gui::ComboOverview, ComboOverview::On; "combos")]
	#[test_case(Gui::Settings, Settings::On; "settings")]
	fn add_gui<TState>(state: Gui, expected: TState) -> Result<(), RunSystemError>
	where
		TState: FreelyMutableState,
	{
		let mut app = setup();

		app.world_mut()
			.run_system_once(move |mut w: GameStatesWriteParam| {
				w.gui_mut().insert(state);
			})?;

		assert_state_eq!(
			&NextState::Pending(expected),
			app.world().resource::<NextState<TState>>()
		);
		Ok(())
	}

	#[test_case(Gui::Hud, Hud::Off; "hud")]
	#[test_case(Gui::Inventory, Inventory::Off; "inventory")]
	#[test_case(Gui::ComboOverview, ComboOverview::Off; "combos")]
	#[test_case(Gui::Settings, Settings::Off; "settings")]
	fn remove_gui<TState>(state: Gui, expected: TState) -> Result<(), RunSystemError>
	where
		TState: FreelyMutableState,
	{
		let mut app = setup();
		app.world_mut()
			.resource_mut::<GameStateContext>()
			.gui
			.insert(state);

		app.world_mut()
			.run_system_once(move |mut w: GameStatesWriteParam| {
				w.gui_mut().remove(&state);
			})?;

		assert_state_eq!(
			&NextState::Pending(expected),
			app.world().resource::<NextState<TState>>()
		);
		Ok(())
	}

	#[test]
	fn no_setter_if_game_state_would_be_unchanged() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut()
			.resource_mut::<GameStateContext>()
			.game_state = StateInternal::active(GameState::Play);

		let no_setter = app
			.world_mut()
			.run_system_once(|mut p: GameStatesWriteParam| {
				p.get_game_state_setter(GameState::Play).is_none()
			})?;

		assert!(no_setter);
		Ok(())
	}

	#[test_case(PhantomData::<Hud>; "hud")]
	#[test_case(PhantomData::<Inventory>; "inventory")]
	#[test_case(PhantomData::<ComboOverview>; "combos")]
	#[test_case(PhantomData::<Settings>; "settings")]
	fn do_nothing_if_gui_would_not_change<TState>(
		_: PhantomData<TState>,
	) -> Result<(), RunSystemError>
	where
		TState: FreelyMutableState,
	{
		let mut app = setup();
		app.world_mut().resource_mut::<GameStateContext>().gui =
			HashSet::from([Gui::Hud, Gui::Settings]);

		app.world_mut()
			.run_system_once(|mut p: GameStatesWriteParam| {
				*p.gui_mut() = HashSet::from([Gui::Hud, Gui::Settings]);
			})?;

		assert_state_eq!(
			&NextState::<TState>::Unchanged,
			app.world().resource::<NextState<TState>>()
		);
		Ok(())
	}

	fn execute_once(
		exec: impl Fn(&mut GameStatesWriteParam),
	) -> impl FnMut(GameStatesWriteParam, Local<bool>) {
		move |mut p: GameStatesWriteParam, mut done: Local<bool>| {
			if *done {
				return;
			}

			exec(&mut p);
			*done = true;
		}
	}

	#[test]
	fn do_not_repeat_stale_game_state_change() -> Result<(), RunSystemError> {
		let mut app = setup();

		app.add_systems(
			Update,
			execute_once(|p| {
				if let Some(s) = p.get_game_state_setter(GameState::Play) {
					s.set_game_state();
				}
			}),
		);
		app.update();
		app.update();

		assert_eq!(&_State(None), app.world().resource::<_State>());
		Ok(())
	}

	#[test_case(Gui::Hud, PhantomData::<Hud>; "hud")]
	#[test_case(Gui::Inventory, PhantomData::<Inventory>; "inventory")]
	#[test_case(Gui::ComboOverview, PhantomData::<ComboOverview>; "combos")]
	#[test_case(Gui::Settings, PhantomData::<Settings>; "settings")]
	fn do_not_repeat_stale_gui_change<T>(
		ui_state: Gui,
		_: PhantomData<T>,
	) -> Result<(), RunSystemError>
	where
		T: FreelyMutableState,
	{
		let mut app = setup();

		app.add_systems(
			Update,
			execute_once(move |p| {
				p.gui_mut().insert(ui_state);
			}),
		);
		app.update();
		app.update();

		assert_state_eq!(
			&NextState::<T>::Unchanged,
			app.world().resource::<NextState<T>>()
		);
		Ok(())
	}
}
