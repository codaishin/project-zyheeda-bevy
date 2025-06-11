use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UI {
	pub background: Entity,
	pub foreground: Entity,
}
