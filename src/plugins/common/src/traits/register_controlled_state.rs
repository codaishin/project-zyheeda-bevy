mod app;

use crate::traits::{automatic_transitions::AutoTransitions, pause_control::PauseControl};

pub trait RegisterControlledState {
	fn register_controlled_state<TState>(&mut self) -> &mut Self
	where
		TState: AutoTransitions + PauseControl + Default + Clone;
}
