use crate::states::state_internal::StateInternal;
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

impl From<GameState> for StateEvent<GameState> {
	fn from(value: GameState) -> Self {
		Self::Active(value)
	}
}

impl<T> From<GameStateExtended<T>> for StateEvent<GameStateExtended<T>>
where
	T: ThreadSafe + Debug + PartialEq + Eq + Hash + Clone + Copy,
{
	fn from(value: GameStateExtended<T>) -> Self {
		Self::Active(value)
	}
}

impl<T> From<StateEvent<T>> for StateInternal<T>
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
