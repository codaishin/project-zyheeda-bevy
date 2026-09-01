use common::prelude::*;

pub(crate) trait TriggerState {
	fn trigger_state(&self) -> GameStateCommand;
}
