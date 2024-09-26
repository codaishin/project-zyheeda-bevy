use std::time::Duration;

pub trait ProcessDelta {
	fn process_delta(&mut self, delta: Duration);
}
