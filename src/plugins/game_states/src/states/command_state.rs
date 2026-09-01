use bevy::prelude::*;
use common::prelude::*;
use std::{fmt::Debug, hash::Hash};

#[derive(States, Debug, PartialEq, Eq, Hash, Default, Clone, Copy)]
pub(crate) enum CommandState<T = GameStateCommand>
where
	T: ThreadSafe + Debug + PartialEq + Eq + Hash + Clone + Copy,
{
	#[default]
	Dirty,
	Active(T),
}

impl From<GameStateCommand> for CommandState {
	fn from(cmd: GameStateCommand) -> Self {
		Self::Active(cmd)
	}
}

impl<T> From<CommandState> for CommandState<GameStateCommandExtended<T>>
where
	T: ThreadSafe + Debug + PartialEq + Eq + Hash + Clone + Copy,
{
	fn from(cmd: CommandState) -> Self {
		match cmd {
			CommandState::Dirty => Self::Dirty,
			CommandState::Active(cmd) => Self::Active(GameStateCommandExtended::Command(cmd)),
		}
	}
}

impl TryFrom<CommandState> for GameStateCommand {
	type Error = IsDirty;

	fn try_from(value: CommandState) -> Result<Self, Self::Error> {
		match value {
			CommandState::Dirty => Err(IsDirty),
			CommandState::Active(cmd) => Ok(cmd),
		}
	}
}

pub(crate) struct IsDirty;
