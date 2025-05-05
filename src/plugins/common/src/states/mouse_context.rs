use crate::tools::keys::user_input::UserInput;
use bevy::prelude::*;
use std::{fmt::Debug, hash::Hash};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, States)]
pub enum MouseContext {
	#[default]
	Default,
	UI,
	Primed(UserInput),
	JustTriggered(UserInput),
	Triggered(UserInput),
	JustReleased(UserInput),
}
