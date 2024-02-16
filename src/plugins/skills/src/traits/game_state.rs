use crate::states::{GameRunning, Off, On};
use common::traits::get_state::GetState;

impl GetState<On> for GameRunning {
	fn get_state() -> Self {
		GameRunning::On
	}
}

impl GetState<Off> for GameRunning {
	fn get_state() -> Self {
		GameRunning::Off
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::traits::get_state::test_tools::get;

	#[test]
	fn turn_on() {
		assert_eq!(GameRunning::On, get::<GameRunning, On>());
	}

	#[test]
	fn turn_off() {
		assert_eq!(GameRunning::Off, get::<GameRunning, Off>());
	}
}
