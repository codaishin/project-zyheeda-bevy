use bevy::prelude::*;

pub(crate) trait MoveTogether {
	fn entity(&self) -> Option<Entity>;
	fn move_together_with(&mut self, transform: &mut Transform, new_position: Vec3);
}
