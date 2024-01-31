use bevy::{ecs::schedule::States, input::keyboard::KeyCode};
use core::hash::Hash;
use std::fmt::Debug;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, States)]
pub enum GameRunning {
	#[default]
	None,
	On,
	Off,
}

pub struct On;

pub struct Off;

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
