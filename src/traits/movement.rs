mod simple_movement;

use bevy::prelude::*;

pub type Units = f32;

pub trait Movement {
	fn update(&mut self, agent: &mut Transform, distance: Units);
}
