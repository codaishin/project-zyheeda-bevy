use bevy::prelude::*;

pub trait Orbit {
	fn orbit(&self, agent: &mut Transform, angles: Vec2Radians);
}

pub type Vec2Radians = Vec2;
