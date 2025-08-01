use bevy::state::state::FreelyMutableState;

pub trait PauseControl: FreelyMutableState {
	fn non_pause_states() -> impl IntoIterator<Item = Self>;
}
