use bevy::{
	ecs::{component::Component, entity::Entity, system::Commands},
	math::Vec3,
	render::color::Color,
};
use common::tools::UnitsPerSecond;
use std::{marker::PhantomData, time::Duration};

#[derive(Component)]
pub struct CamOrbit {
	pub center: Vec3,
	pub distance: f32,
	pub sensitivity: f32,
}

pub struct Plasma;

#[derive(Component, Default)]
pub struct Projectile<T> {
	pub direction: Vec3,
	pub range: f32,
	phantom_data: PhantomData<T>,
}

impl<T> Projectile<T> {
	pub fn new(direction: Vec3, range: f32) -> Self {
		Self {
			direction,
			range,
			phantom_data: PhantomData,
		}
	}
}

#[derive(Component)]
pub struct VoidSphere;

impl VoidSphere {
	pub const AGGRO_RANGE: f32 = 10.;
	pub const ATTACK_RANGE: f32 = 5.;
}

#[derive(Component, Clone)]
pub enum VoidSpherePart {
	Core,
	RingA(UnitsPerSecond),
	RingB(UnitsPerSecond),
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Attacker(pub Entity);

#[derive(Debug, PartialEq, Clone, Copy)]
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
	pub aggro_range: f32,
	pub attack_range: f32,
	pub foe: Foe,
}

#[derive(Component, Clone, Copy)]
pub struct Beam {
	pub target: Entity,
	pub range: f32,
}

impl Beam {
	pub fn attack(commands: &mut Commands, attacker: Attacker, target: Target) {
		commands.entity(attacker.0).insert(Beam {
			target: target.0,
			range: VoidSphere::ATTACK_RANGE,
		});
	}
}

#[derive(Component, Debug, PartialEq)]
pub(crate) struct ActiveBeam {
	pub from: Vec3,
	pub to: Vec3,
	pub color: Color,
	pub emission: Color,
}
