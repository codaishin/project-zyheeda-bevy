use bevy::prelude::*;

#[derive(States, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) struct MenusChangeable(pub(crate) bool);

impl Default for MenusChangeable {
	fn default() -> Self {
		MenusChangeable(true)
	}
}
