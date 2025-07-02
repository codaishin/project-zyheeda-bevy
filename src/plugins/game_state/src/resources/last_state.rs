use bevy::{prelude::*, state::state::FreelyMutableState};

#[derive(Resource, Debug, PartialEq, Default)]
pub(crate) struct LastState<TState>(Option<TState>)
where
	TState: FreelyMutableState;
