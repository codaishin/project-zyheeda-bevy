use bevy::prelude::*;

#[derive(Message, Debug, PartialEq)]
pub(crate) enum DropdownMessage {
	Added(Entity),
	Removed(Entity),
}
