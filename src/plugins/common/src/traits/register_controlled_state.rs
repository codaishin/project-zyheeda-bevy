mod app;

use crate::traits::{automatic_transitions::AutoTransitions, pause_control::PauseControl};

pub trait RegisterControlledState {
	/// Register the state, so that initializations, auto transitions and pause control are executed.
	fn register_controlled_state<TState>(&mut self) -> &mut Self
	where
		TState: AutoTransitions + PauseControl + Default + Clone;
}
