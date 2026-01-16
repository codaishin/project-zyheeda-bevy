use bevy::prelude::*;

pub(crate) trait SendCollisionInteraction {
	fn start_interaction(&mut self, a: Entity, b: Entity);
	fn end_interaction(&mut self, a: Entity, b: Entity);
}
