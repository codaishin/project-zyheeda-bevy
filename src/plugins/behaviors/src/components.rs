use bevy::{ecs::component::Component, math::Vec3};
use common::tools::UnitsPerSecond;

#[derive(Component)]
pub struct CamOrbit {
	pub center: Vec3,
	pub distance: f32,
	pub sensitivity: f32,
}

#[derive(Component, Clone, Copy, PartialEq, Debug)]
pub struct SimpleMovement {
	pub target: Vec3,
}

impl SimpleMovement {
	pub fn new(target: Vec3) -> Self {
		Self { target }
	}
}

#[derive(Component)]
pub struct Idle;

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum MovementMode {
	#[default]
	Fast,
	Slow,
}

#[derive(Component)]
pub enum MovementConfig {
	Constant {
		mode: MovementMode,
		speed: UnitsPerSecond,
	},
	Dynamic {
		current_mode: MovementMode,
		fast_speed: UnitsPerSecond,
		slow_speed: UnitsPerSecond,
	},
}
