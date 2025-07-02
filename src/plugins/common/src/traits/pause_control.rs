use bevy::state::state::FreelyMutableState;

pub trait PauseControl: FreelyMutableState {
	fn pause_transitions() -> &'static [OnTransition<Self>];
	fn unpause_transitions() -> &'static [OnTransition<Self>];
}

pub enum OnTransition<TState> {
	Enter(TState),
	Exit(TState),
}
