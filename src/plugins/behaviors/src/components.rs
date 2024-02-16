use bevy::{
	ecs::{component::Component, entity::Entity, system::Commands},
	math::Vec3,
};
use common::tools::UnitsPerSecond;
use std::time::Duration;

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

#[derive(Component, Debug, PartialEq)]
pub(crate) struct Attack(pub Entity);

#[derive(Debug, PartialEq)]
pub struct Attacker(pub Entity);

#[derive(Debug, PartialEq)]
pub struct Target(pub Entity);

#[derive(Component)]
pub struct AttackConfig {
	pub attack: fn(&mut Commands, Attacker, Target),
	pub cool_down: Duration,
}

#[derive(Component, Debug, PartialEq)]
pub(crate) struct OnCoolDown(pub Duration);

#[derive(Clone, Copy)]
pub enum Foe {
	Player,
	Entity(Entity),
}

#[derive(Component, Clone, Copy)]
pub struct Enemy {
	pub attack_range: f32,
	pub aggro_range: f32,
	pub foe: Foe,
}
