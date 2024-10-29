pub mod cam_orbit;
pub mod ground_targeted_aoe;
pub mod projectile;
pub mod shield;

use crate::traits::{RemoveComponent, SpawnAttack};
use bevy::{ecs::system::EntityCommands, prelude::*};
use common::{test_tools::utils::ApproxEqual, tools::UnitsPerSecond};
use std::{fmt::Debug, marker::PhantomData, sync::Arc, time::Duration};

#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub enum Face {
	#[default]
	Cursor,
	Entity(Entity),
	Translation(Vec3),
}

#[derive(Component, Debug, PartialEq)]
pub struct OverrideFace(pub Face);

#[derive(Component, Debug, PartialEq)]
pub struct SetFace(pub Face);

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

#[derive(PartialEq, Debug)]
pub struct PositionBased;

#[derive(PartialEq, Debug)]
pub struct VelocityBased;

#[derive(Component, Clone, PartialEq, Debug, Default)]
pub struct Movement<TMovement> {
	pub(crate) target: Vec3,
	pub(crate) cleanup: Option<fn(&mut EntityCommands)>,
	phantom_data: PhantomData<TMovement>,
}

impl<TMovement> Movement<TMovement> {
	pub fn to(target: Vec3) -> Self {
		Self {
			target,
			cleanup: None,
			phantom_data: PhantomData,
		}
	}

	pub fn remove_on_cleanup<TBundle: Bundle>(self) -> Self {
		Self {
			target: self.target,
			cleanup: Some(TBundle::get_remover()),
			phantom_data: self.phantom_data,
		}
	}
}

impl<TMovement> ApproxEqual<f32> for Movement<TMovement> {
	fn approx_equal(&self, other: &Self, tolerance: &f32) -> bool {
		self.target.approx_equal(&other.target, tolerance) && self.cleanup == other.cleanup
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
pub struct Chase(pub Entity);

#[derive(Component, Debug, PartialEq)]
pub struct Attack(pub Entity);

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Attacker(pub Entity);

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Target(pub Entity);

#[derive(Component)]
pub struct AttackConfig {
	pub spawn: Arc<dyn SpawnAttack + Sync + Send>,
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

#[derive(Component, Debug, PartialEq, Default)]
pub struct LifeTime(pub Duration);

#[derive(Component, Default, Clone, Copy, Debug, PartialEq)]
pub struct BeamConfig {
	pub damage: f32,
	pub color: Color,
	pub emissive: LinearRgba,
	pub lifetime: Duration,
	pub range: f32,
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct BeamCommand {
	pub source: Entity,
	pub target: Entity,
}

#[derive(Component, Default, Debug, PartialEq)]
pub(crate) struct Beam {
	pub damage: f32,
	pub from: Vec3,
	pub to: Vec3,
	pub color: Color,
	pub emissive: LinearRgba,
}
