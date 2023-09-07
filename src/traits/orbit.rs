use bevy::prelude::*;

pub type Vec2Radians = Vec2;

pub trait Orbit {
	fn orbit(&self, agent: &mut Transform, angles: Vec2Radians);
}
