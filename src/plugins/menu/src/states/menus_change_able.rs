use bevy::prelude::*;

#[derive(States, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) struct MenusChangAble(pub(crate) bool);

impl Default for MenusChangAble {
	fn default() -> Self {
		MenusChangAble(true)
	}
}
