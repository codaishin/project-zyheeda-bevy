pub mod game_state;
pub mod load_state;
pub mod menu_state;

use bevy::{input::keyboard::KeyCode, prelude::*};
use std::{fmt::Debug, hash::Hash};

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
