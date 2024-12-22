use bevy::prelude::EntityCommands;
use common::traits::handles_enemies::{Attacker, Target};

pub trait InsertAttack {
	fn insert_attack(&self, entity: &mut EntityCommands, attacker: Attacker, target: Target);
}
