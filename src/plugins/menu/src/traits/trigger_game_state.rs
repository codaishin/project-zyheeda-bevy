use bevy::state::state::FreelyMutableState;

pub(crate) trait TriggerState {
	type TState: FreelyMutableState + Clone;

	fn trigger_state(&self) -> Self::TState;
}
