use bevy::{prelude::*, state::state::FreelyMutableState};

#[derive(Resource, Debug, PartialEq)]
pub(crate) struct LastState<TState>(pub(crate) Option<TState>)
where
	TState: FreelyMutableState;

impl<TState> Default for LastState<TState>
where
	TState: FreelyMutableState,
{
	fn default() -> Self {
		Self(None)
	}
}
