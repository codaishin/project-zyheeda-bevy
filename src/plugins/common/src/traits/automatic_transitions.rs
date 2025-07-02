use bevy::state::state::FreelyMutableState;

pub trait AutoTransitions: FreelyMutableState {
	fn auto_transitions() -> &'static [(Self, TransitionTo<Self>)];
}

pub enum TransitionTo<TState> {
	State(TState),
	PreviousState,
}
