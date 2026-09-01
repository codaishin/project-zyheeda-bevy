use crate::states::command_state::CommandState;
use bevy::prelude::*;
use common::prelude::*;
use std::{fmt::Debug, hash::Hash};

#[derive(Debug, PartialEq, Event)]
pub(crate) struct StateEvent<TCommand>
where
	TCommand: ThreadSafe + Debug + PartialEq + Eq + Hash + Clone + Copy,
{
	pub(crate) state: CommandState<TCommand>,
}
