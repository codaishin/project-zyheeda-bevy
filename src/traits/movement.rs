mod simple_movement;

use bevy::prelude::*;

pub type Seconds = f32;

pub trait Movement {
	fn move_towards(&self, agent: &mut Transform, target: Vec3, delta_time: Seconds);
}
