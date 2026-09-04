use crate::{
	resources::pause_control::Pausable,
	states::state_internal::StateInternal,
	system_params::gui_states::GuiStates,
};
use bevy::{ecs::system::RunSystemOnce, prelude::*};
use common::prelude::*;
use std::collections::HashSet;

#[derive(Resource)]
pub struct GameStateContext {
	pub(crate) game_state: StateInternal<GameState>,
	pub(crate) gui: HashSet<Gui>,
}

impl FromWorld for GameStateContext {
	fn from_world(world: &mut World) -> Self {
		Self {
			game_state: *world.resource::<State<StateInternal<GameState>>>().get(),
			gui: world
				.run_system_once(|p: GuiStates| HashSet::from(&p))
				.unwrap_or_default(),
		}
	}
}

impl<'w> IntoIterator for &'w GameStateContext {
	type Item = Pausable;
	type IntoIter = GameStatesIter<'w>;

	fn into_iter(self) -> Self::IntoIter {
		GameStatesIter {
			command: self.game_state.try_into_active(),
			ui: self.gui.iter(),
		}
	}
}

pub struct GameStatesIter<'w> {
	command: Option<GameState>,
	ui: std::collections::hash_set::Iter<'w, Gui>,
}

impl Iterator for GameStatesIter<'_> {
	type Item = Pausable;

	fn next(&mut self) -> Option<Self::Item> {
		match self.command.take() {
			Some(s) => Some(Pausable::GameState(s)),
			None => self.ui.next().map(|s| Pausable::Gui(*s)),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::states::state_internal::StateInternal;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use std::collections::HashSet;
	use testing::SingleThreadedApp;

	fn setup(ctx: GameStateContext) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(ctx);

		app
	}

	#[test]
	fn iter() -> Result<(), RunSystemError> {
		let mut app = setup(GameStateContext {
			game_state: StateInternal::active(GameState::Save),
			gui: HashSet::from([Gui::ComboOverview, Gui::Inventory]),
		});

		let states = app
			.world_mut()
			.run_system_once(|p: Res<GameStateContext>| p.into_iter().collect::<HashSet<_>>())?;

		assert_eq!(
			HashSet::from([
				Pausable::GameState(GameState::Save),
				Pausable::Gui(Gui::ComboOverview),
				Pausable::Gui(Gui::Inventory)
			]),
			states
		);
		Ok(())
	}
}
