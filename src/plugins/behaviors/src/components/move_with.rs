use bevy::prelude::*;

#[derive(Component, Debug, PartialEq, Clone, Copy)]
pub(crate) struct MoveWith {
	pub(crate) entity: Entity,
}
