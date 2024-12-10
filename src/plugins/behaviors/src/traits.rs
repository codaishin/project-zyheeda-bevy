pub(crate) mod bundle;
pub(crate) mod movement;

use crate::components::{Attacker, Target};
use bevy::{ecs::system::EntityCommands, prelude::*};
use common::{tools::Units, traits::handles_behaviors::Speed};
use std::sync::Arc;

pub type Vec2Radians = Vec2;

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub(crate) struct IsDone(pub(crate) bool);

pub(crate) trait Caster {
	fn caster(&self) -> Entity;
}

pub(crate) trait Spawner {
	fn spawner(&self) -> Entity;
}

pub(crate) trait ProjectileBehavior: Caster {
	fn range(&self) -> f32;
}

pub trait Orbit {
	fn orbit(&self, agent: &mut Transform, angles: Vec2Radians);
}

pub(crate) trait MoveTogether {
	fn entity(&self) -> Option<Entity>;
	fn move_together_with(&mut self, transform: &mut Transform, new_position: Vec3);
}

pub(crate) trait MovementPositionBased {
	fn update(&mut self, agent: &mut Transform, distance: Units) -> IsDone;
}

pub(crate) trait MovementVelocityBased {
	fn update(&self, agent: &mut EntityCommands, position: Vec3, speed: Speed) -> IsDone;
}

pub(crate) trait Cleanup {
	fn cleanup(&self, agent: &mut EntityCommands);
}

pub type DespawnFn = Arc<dyn Fn(&mut Commands) + Sync + Send>;

pub trait SpawnAttack {
	fn spawn(&self, commands: &mut Commands, attacker: Attacker, target: Target) -> DespawnFn;
}

pub trait GetAttackSpawner<TEnemy> {
	fn attack_spawner(enemy: &TEnemy) -> Arc<dyn SpawnAttack>;
}

pub trait RemoveComponent<T: Bundle> {
	fn get_remover() -> fn(&mut EntityCommands);
}
