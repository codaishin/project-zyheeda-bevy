pub(crate) mod beam;
pub(crate) mod beam_config;
pub(crate) mod bundle;
pub(crate) mod cam_orbit;
pub(crate) mod force_shield;
pub(crate) mod movement;
pub(crate) mod movement_config;
pub(crate) mod void_sphere;

use crate::components::{Attacker, MovementMode, Target};
use bevy::{
	ecs::{
		bundle::Bundle,
		system::{Commands, EntityCommands},
	},
	math::{Dir3, Vec2, Vec3},
	transform::components::Transform,
};
use common::tools::{Units, UnitsPerSecond};
use std::sync::Arc;

pub type Vec2Radians = Vec2;

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub(crate) struct IsDone(bool);

impl IsDone {
	pub fn is_done(&self) -> bool {
		self.0
	}
}

impl From<bool> for IsDone {
	fn from(value: bool) -> Self {
		Self(value)
	}
}

pub(crate) trait ProjectileBehavior {
	fn direction(&self) -> Dir3;
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

pub(crate) trait MovementPositionBased {
	fn update(&mut self, agent: &mut Transform, distance: Units) -> IsDone;
}

pub(crate) trait MovementVelocityBased {
	fn update(&self, agent: &mut EntityCommands, position: Vec3, speed: UnitsPerSecond) -> IsDone;
}

pub(crate) trait Cleanup {
	fn cleanup(&self, agent: &mut EntityCommands);
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

pub trait RemoveComponent<T: Bundle> {
	fn get_remover() -> fn(&mut EntityCommands);
}

pub(crate) trait GetAnimation<TAnimation> {
	fn animation<'s>(&'s self, key: &MovementMode) -> &'s TAnimation;
}
