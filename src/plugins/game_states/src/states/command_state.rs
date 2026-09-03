use bevy::prelude::*;
use common::prelude::*;
use std::{fmt::Debug, hash::Hash};

#[derive(States, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) struct CommandState<T = GameStateCommand>(Variant<T>)
where
	T: ThreadSafe + Debug + PartialEq + Eq + Hash + Clone + Copy;

impl<T> CommandState<T>
where
	T: ThreadSafe + Debug + PartialEq + Eq + Hash + Clone + Copy,
{
	pub(crate) fn active(value: T) -> Self {
		Self(Variant::Active(value))
	}

	pub(crate) fn dirty() -> Self {
		Self(Variant::Dirty)
	}

	#[cfg(test)]
	pub(crate) fn none() -> Self {
		Self(Variant::None)
	}

	pub(crate) fn try_into_active(self) -> Option<T> {
		match self.0 {
			Variant::Active(v) => Some(v),
			_ => None,
		}
	}

	pub(crate) fn in_state(
		state: Option<T>,
	) -> impl IntoSystem<(), bool, (), System: ReadOnlySystem> {
		IntoSystem::into_system(
			move |current: Res<State<Self>>| match (&state, current.get()) {
				(Some(state), Self(Variant::Active(current))) => state == current,
				(None, Self(Variant::None)) => true,
				_ => false,
			},
		)
	}
}

impl<T> FromWorld for CommandState<T>
where
	T: ThreadSafe + Debug + PartialEq + Eq + Hash + Clone + Copy,
{
	fn from_world(_: &mut World) -> Self {
		Self(Variant::None)
	}
}

impl From<GameStateCommand> for CommandState {
	fn from(value: GameStateCommand) -> Self {
		Self::active(value)
	}
}

#[derive(States, Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum Variant<T>
where
	T: ThreadSafe + Debug + PartialEq + Eq + Hash + Clone + Copy,
{
	None,
	Dirty,
	Active(T),
}
