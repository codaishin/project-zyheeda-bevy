use bevy::{ecs::component::Component, math::Vec3};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayerMovement {
	Walk,
	Run,
}
