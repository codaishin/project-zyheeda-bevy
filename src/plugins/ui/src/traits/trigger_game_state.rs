use common::traits::handles_game_states::SettableActivity;

pub(crate) trait TriggerState {
	fn trigger_state(&self) -> SettableActivity;
}
