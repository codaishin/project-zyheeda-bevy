use bevy::prelude::{Entity, Event};

#[derive(Event, Debug, PartialEq)]
pub(crate) enum DropdownEvent {
	Added(Entity),
	Removed(Entity),
}
