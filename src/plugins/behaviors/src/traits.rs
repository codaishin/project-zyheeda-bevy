pub(crate) mod beam;
pub(crate) mod beam_config;
pub(crate) mod cam_orbit;
pub(crate) mod movement_config;
pub(crate) mod projectile;
pub(crate) mod simple_movement;
pub(crate) mod void_sphere;

use crate::components::{Attacker, MovementMode, Target};
use bevy::{
	ecs::system::Commands,
	math::{Vec2, Vec3},
	transform::components::Transform,
};
use common::tools::UnitsPerSecond;
use std::sync::Arc;

pub(crate) type Units = f32;
pub(crate) type IsDone = bool;
pub type Vec2Radians = Vec2;

pub(crate) trait ProjectileBehavior {
	fn direction(&self) -> Vec3;
	fn range(&self) -> f32;
}

pub(crate) trait MovementData {
	fn get_movement_data(&self) -> (UnitsPerSecond, MovementMode);
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

type DespawnFn = Arc<dyn Fn(&mut Commands) + Sync + Send>;

pub trait SpawnAttack {
	fn spawn(&self, commands: &mut Commands, attacker: Attacker, target: Target) -> DespawnFn;
}

pub trait ToArc {
	fn to_arc(self) -> Arc<Self>;
}

impl<T: SpawnAttack> ToArc for T {
	fn to_arc(self) -> Arc<Self> {
		Arc::new(self)
	}
}
