pub(crate) mod cam_orbit;
pub(crate) mod player;
pub(crate) mod projectile;
pub(crate) mod simple_movement;
pub(crate) mod void_sphere;

use bevy::{
	math::{Vec2, Vec3},
	transform::components::Transform,
};
use common::{components::Animate, tools::UnitsPerSecond};

pub(crate) type Units = f32;
pub(crate) type IsDone = bool;
pub type Vec2Radians = Vec2;

pub(crate) trait ProjectileBehavior {
	fn direction(&self) -> Vec3;
	fn range(&self) -> f32;
}

pub(crate) trait MovementData<TAnimationKey: Clone + Copy> {
	fn get_movement_data(&self) -> (UnitsPerSecond, Animate<TAnimationKey>);
}

pub trait Orbit {
	fn orbit(&self, agent: &mut Transform, angles: Vec2Radians);
}

pub(crate) trait MoveTogether {
	fn move_together_with(&mut self, transform: &mut Transform, new_position: Vec3);
}

pub(crate) trait Movement {
	fn update(&mut self, agent: &mut Transform, distance: Units) -> IsDone;
}
