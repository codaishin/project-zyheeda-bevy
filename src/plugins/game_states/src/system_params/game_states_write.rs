use crate::{
	events::StateEvent,
	resources::game_state_context::GameStateContext,
	system_params::ui_states::UIStatesMut,
};
use bevy::{ecs::system::SystemParam, prelude::*};
use common::prelude::{SetGameState as SetGameStateTrait, *};
use std::collections::HashSet;

#[derive(SystemParam)]
pub struct GameStatesWriteParam<'w, 's> {
	current: Res<'w, GameStateContext>,
	activity: ActivityStateMut<'w, 's>,
	ui: UIStatesMut<'w>,
	command_change: Local<'s, Option<GameStateCommand>>,
	ui_change: Local<'s, Option<HashSet<IngameUI>>>,
}

impl GameStatesWriteParam<'_, '_> {
	fn drain_activity_change(&mut self) {
		let Some(activity) = self.command_change.take() else {
			return;
		};

		self.activity.set(activity);
	}

	fn drain_ui_change(&mut self) {
		let Some(ui) = self.ui_change.take() else {
			return;
		};

		let detect_change = |state| Change::of(state, &self.current.ui, &ui);
		let changes = IngameUI::iterator().filter_map(detect_change);

		for change in changes {
			match change {
				Change::Added(state) => self.ui.set_on(state),
				Change::Removed(state) => self.ui.set_off(&state),
			}
		}
	}
}

impl GameStates for GameStatesWriteParam<'_, '_> {
	fn command(&self) -> Option<GameStateCommand> {
		self.current.command_state.try_into_active()
	}

	fn ui(&self) -> &'_ HashSet<IngameUI> {
		&self.current.ui
	}
}

impl GameStatesMut for GameStatesWriteParam<'_, '_> {
	type TGameStateSetter<'a>
		= SetGameState<'a>
	where
		Self: 'a;

	fn get_game_state_setter(&mut self, new: GameStateCommand) -> Option<SetGameState<'_>> {
		let command = match (self.current.command_state.try_into_active(), new) {
			(Some(current), new) if current != new => new,
			(Some(GameStateCommand::Pause), GameStateCommand::Pause) => GameStateCommand::Play,
			_ => return None,
		};

		Some(SetGameState {
			command,
			change: &mut self.command_change,
		})
	}

	fn ui_mut(&mut self) -> &'_ mut HashSet<IngameUI> {
		self.ui_change.get_or_insert(self.current.ui.clone())
	}
}

impl Drop for GameStatesWriteParam<'_, '_> {
	fn drop(&mut self) {
		self.drain_activity_change();
		self.drain_ui_change();
	}
}

pub struct SetGameState<'a> {
	command: GameStateCommand,
	change: &'a mut Option<GameStateCommand>,
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
	fn set(&mut self, state: impl Into<StateEvent<GameStateCommand>>) {
		self.commands.trigger_observers_for(state.into());
	}
}

enum Change {
	Added(IngameUI),
	Removed(IngameUI),
}

impl Change {
	fn of(
		state: IngameUI,
		current: &HashSet<IngameUI>,
		queued: &HashSet<IngameUI>,
	) -> Option<Change> {
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
			command_state::CommandState,
			ui::{ComboOverview, Hud, Inventory, Settings},
		},
		system_params::ui_states::UIStates,
	};
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		state::{app::StatesPlugin, state::FreelyMutableState},
	};
	use std::marker::PhantomData;
	use test_case::test_case;
	use testing::SingleThreadedApp;

	#[derive(Resource, Debug, PartialEq)]
	struct _State(Option<StateEvent<GameStateCommand>>);

	impl _State {
		fn record(on_state: On<StateEvent<GameStateCommand>>, mut c: Commands) {
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
		UIStates::init(&mut app);
		app.insert_resource(_State(None));
		app.init_state::<CommandState>();
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
	fn set_activity() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut()
			.resource_mut::<GameStateContext>()
			.command_state = CommandState::active(GameStateCommand::NewGame);

		app.world_mut()
			.run_system_once(move |mut w: GameStatesWriteParam| {
				if let Some(s) = w.get_game_state_setter(GameStateCommand::Play) {
					s.set_game_state()
				};
			})?;

		assert_eq!(
			&_State(Some(StateEvent::Active(GameStateCommand::Play))),
			app.world().resource::<_State>()
		);
		Ok(())
	}

	#[test]
	fn set_activity_paused_to_play() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut()
			.resource_mut::<GameStateContext>()
			.command_state = CommandState::active(GameStateCommand::Pause);

		app.world_mut()
			.run_system_once(move |mut w: GameStatesWriteParam| {
				if let Some(s) = w.get_game_state_setter(GameStateCommand::Pause) {
					s.set_game_state()
				};
			})?;

		assert_eq!(
			&_State(Some(StateEvent::Active(GameStateCommand::Play))),
			app.world().resource::<_State>()
		);
		Ok(())
	}

	#[test]
	fn set_activity_play_to_paused() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut()
			.resource_mut::<GameStateContext>()
			.command_state = CommandState::active(GameStateCommand::Play);

		app.world_mut()
			.run_system_once(move |mut w: GameStatesWriteParam| {
				if let Some(s) = w.get_game_state_setter(GameStateCommand::Pause) {
					s.set_game_state()
				};
			})?;

		assert_eq!(
			&_State(Some(StateEvent::Active(GameStateCommand::Pause))),
			app.world().resource::<_State>()
		);
		Ok(())
	}

	#[test_case(IngameUI::Hud, Hud::On; "hud")]
	#[test_case(IngameUI::Inventory, Inventory::On; "inventory")]
	#[test_case(IngameUI::ComboOverview, ComboOverview::On; "combos")]
	#[test_case(IngameUI::Settings, Settings::On; "settings")]
	fn add_ui<TState>(state: IngameUI, expected: TState) -> Result<(), RunSystemError>
	where
		TState: FreelyMutableState,
	{
		let mut app = setup();

		app.world_mut()
			.run_system_once(move |mut w: GameStatesWriteParam| {
				w.ui_mut().insert(state);
			})?;

		assert_state_eq!(
			&NextState::Pending(expected),
			app.world().resource::<NextState<TState>>()
		);
		Ok(())
	}

	#[test_case(IngameUI::Hud, Hud::Off; "hud")]
	#[test_case(IngameUI::Inventory, Inventory::Off; "inventory")]
	#[test_case(IngameUI::ComboOverview, ComboOverview::Off; "combos")]
	#[test_case(IngameUI::Settings, Settings::Off; "settings")]
	fn remove_ui<TState>(state: IngameUI, expected: TState) -> Result<(), RunSystemError>
	where
		TState: FreelyMutableState,
	{
		let mut app = setup();
		app.world_mut()
			.resource_mut::<GameStateContext>()
			.ui
			.insert(state);

		app.world_mut()
			.run_system_once(move |mut w: GameStatesWriteParam| {
				w.ui_mut().remove(&state);
			})?;

		assert_state_eq!(
			&NextState::Pending(expected),
			app.world().resource::<NextState<TState>>()
		);
		Ok(())
	}

	#[test]
	fn no_setter_if_activity_would_be_unchanged() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut()
			.resource_mut::<GameStateContext>()
			.command_state = CommandState::active(GameStateCommand::Play);

		let no_setter = app
			.world_mut()
			.run_system_once(|mut p: GameStatesWriteParam| {
				p.get_game_state_setter(GameStateCommand::Play).is_none()
			})?;

		assert!(no_setter);
		Ok(())
	}

	#[test_case(PhantomData::<Hud>; "hud")]
	#[test_case(PhantomData::<Inventory>; "inventory")]
	#[test_case(PhantomData::<ComboOverview>; "combos")]
	#[test_case(PhantomData::<Settings>; "settings")]
	fn do_nothing_if_ui_would_ne_unchanged<TState>(
		_: PhantomData<TState>,
	) -> Result<(), RunSystemError>
	where
		TState: FreelyMutableState,
	{
		let mut app = setup();
		app.world_mut().resource_mut::<GameStateContext>().ui =
			HashSet::from([IngameUI::Hud, IngameUI::Settings]);

		app.world_mut()
			.run_system_once(|mut p: GameStatesWriteParam| {
				*p.ui_mut() = HashSet::from([IngameUI::Hud, IngameUI::Settings]);
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
	fn do_not_repeat_stale_activity_change() -> Result<(), RunSystemError> {
		let mut app = setup();

		app.add_systems(
			Update,
			execute_once(|p| {
				if let Some(s) = p.get_game_state_setter(GameStateCommand::Play) {
					s.set_game_state();
				}
			}),
		);
		app.update();
		app.update();

		assert_eq!(&_State(None), app.world().resource::<_State>());
		Ok(())
	}

	#[test_case(IngameUI::Hud, PhantomData::<Hud>; "hud")]
	#[test_case(IngameUI::Inventory, PhantomData::<Inventory>; "inventory")]
	#[test_case(IngameUI::ComboOverview, PhantomData::<ComboOverview>; "combos")]
	#[test_case(IngameUI::Settings, PhantomData::<Settings>; "settings")]
	fn do_not_repeat_stale_ui_change<T>(
		ui_state: IngameUI,
		_: PhantomData<T>,
	) -> Result<(), RunSystemError>
	where
		T: FreelyMutableState,
	{
		let mut app = setup();

		app.add_systems(
			Update,
			execute_once(move |p| {
				p.ui_mut().insert(ui_state);
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
