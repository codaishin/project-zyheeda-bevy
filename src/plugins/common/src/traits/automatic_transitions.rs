use bevy::state::state::FreelyMutableState;

pub trait AutomaticTransitions: FreelyMutableState {
	fn transitions() -> &'static [(Self, TransitionTo<Self>)];
}

pub enum TransitionTo<TState> {
	State(TState),
	PreviousState,
}
