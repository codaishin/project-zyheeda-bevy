mod simple_movement;

use bevy::prelude::*;

pub type Seconds = f32;

pub trait Movement {
	fn update(&mut self, agent: &mut Transform, delta_time: Seconds);
}
