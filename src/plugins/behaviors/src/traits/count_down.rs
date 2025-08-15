use std::time::Duration;

pub(crate) trait CountDown: Sized {
	fn remaining_mut(&mut self) -> &mut Duration;
	fn next_state(&self) -> Option<Self>;
}
