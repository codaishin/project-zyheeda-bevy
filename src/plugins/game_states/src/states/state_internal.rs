use bevy::prelude::*;
use common::prelude::*;
use std::{fmt::Debug, hash::Hash};

#[derive(States, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) struct StateInternal<T>(Variant<T>)
where
	T: ThreadSafe + Debug + PartialEq + Eq + Hash + Clone + Copy;

impl<T> StateInternal<T>
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

impl<T> FromWorld for StateInternal<T>
where
	T: ThreadSafe + Debug + PartialEq + Eq + Hash + Clone + Copy,
{
	fn from_world(_: &mut World) -> Self {
		Self(Variant::None)
	}
}

impl From<GameState> for StateInternal<GameState> {
	fn from(value: GameState) -> Self {
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
