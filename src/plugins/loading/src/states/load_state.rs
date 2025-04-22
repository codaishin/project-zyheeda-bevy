use bevy::prelude::*;
use common::traits::{
	handles_load_tracking::{IsProcessing, Progress},
	thread_safe::ThreadSafe,
};
use std::{fmt::Debug, hash::Hash, marker::PhantomData};

#[derive(States)]
pub(crate) struct Load<TLoadGroup>
where
	TLoadGroup: ThreadSafe,
{
	_p: PhantomData<TLoadGroup>,
	state: State,
}

impl<TLoadGroup> Load<TLoadGroup>
where
	TLoadGroup: ThreadSafe,
{
	pub(crate) fn new(state: State) -> Self {
		Self {
			_p: PhantomData,
			state,
		}
	}

	pub(crate) fn processing<TProgress>() -> Self
	where
		TProgress: Progress,
	{
		match TProgress::IS_PROCESSING {
			IsProcessing::Assets => Self::new(State::LoadAssets),
			IsProcessing::Dependencies => Self::new(State::ResolveDependencies),
		}
	}
}

impl<TLoadGroup> Debug for Load<TLoadGroup>
where
	TLoadGroup: ThreadSafe,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Load")
			.field("_p", &self._p)
			.field("state", &self.state)
			.finish()
	}
}

impl<TLoadGroup> Default for Load<TLoadGroup>
where
	TLoadGroup: ThreadSafe,
{
	fn default() -> Self {
		Self {
			_p: PhantomData,
			state: State::default(),
		}
	}
}

impl<TLoadGroup> PartialEq for Load<TLoadGroup>
where
	TLoadGroup: ThreadSafe,
{
	fn eq(&self, other: &Self) -> bool {
		self.state == other.state
	}
}

impl<TLoadGroup> Eq for Load<TLoadGroup> where TLoadGroup: ThreadSafe {}

impl<TLoadGroup> Hash for Load<TLoadGroup>
where
	TLoadGroup: ThreadSafe,
{
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.state.hash(state);
	}
}

impl<TLoadGroup> Clone for Load<TLoadGroup>
where
	TLoadGroup: ThreadSafe,
{
	fn clone(&self) -> Self {
		*self
	}
}

impl<TLoadGroup> Copy for Load<TLoadGroup> where TLoadGroup: ThreadSafe {}

#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash, Default)]
pub(crate) enum State {
	#[default]
	Idle,
	LoadAssets,
	ResolveDependencies,
	Done,
}
