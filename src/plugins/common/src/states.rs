use crate::traits::get_state::GetState;
use bevy::{input::keyboard::KeyCode, prelude::States};
use std::{fmt::Debug, hash::Hash};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, States)]
pub enum GameRunning {
	#[default]
	None,
	On,
	Off,
}

pub struct On;

pub struct Off;

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

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, States)]
pub enum MouseContext<TKey = KeyCode>
where
	TKey: Debug + Hash + Eq + Clone + Sync + Send + 'static,
{
	#[default]
	Default,
	UI,
	Primed(TKey),
	JustTriggered(TKey),
	Triggered(TKey),
	JustReleased(TKey),
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::get_state::test_tools::get;

	#[test]
	fn turn_on() {
		assert_eq!(GameRunning::On, get::<GameRunning, On>());
	}

	#[test]
	fn turn_off() {
		assert_eq!(GameRunning::Off, get::<GameRunning, Off>());
	}
}
