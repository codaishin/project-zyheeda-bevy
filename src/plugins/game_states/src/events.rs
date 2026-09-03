use crate::states::command_state::CommandState;
use bevy::prelude::*;
use common::prelude::*;
use std::{any::TypeId, fmt::Debug, hash::Hash};

#[derive(Debug, PartialEq, Event, Clone, Copy)]
pub(crate) enum StateEvent<T>
where
	T: ThreadSafe + Debug + PartialEq + Eq + Hash + Clone + Copy,
{
	Dirty { issued_by: TypeId },
	Active(T),
}

impl<T> From<StateEvent<T>> for CommandState<T>
where
	T: ThreadSafe + Debug + PartialEq + Eq + Hash + Clone + Copy,
{
	fn from(value: StateEvent<T>) -> Self {
		match value {
			StateEvent::Dirty { .. } => Self::dirty(),
			StateEvent::Active(v) => Self::active(v),
		}
	}
}
