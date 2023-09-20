mod cam_orbit;

use bevy::prelude::{Transform, Vec3};

pub trait MoveTogether {
	fn move_together_with(&mut self, transform: &mut Transform, new_position: Vec3);
}
