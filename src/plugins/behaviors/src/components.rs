use bevy::{
	ecs::{component::Component, entity::Entity},
	math::Vec3,
};
use common::tools::UnitsPerSecond;

#[derive(Component)]
pub struct CamOrbit {
	pub center: Vec3,
	pub distance: f32,
	pub sensitivity: f32,
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum MovementMode {
	#[default]
	Fast,
	Slow,
}

#[derive(Component, Clone, Copy)]
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

#[derive(Component, Debug, PartialEq)]
pub(crate) struct Chase(pub Entity);


#[derive(Component)]
pub struct FollowPlayer;
