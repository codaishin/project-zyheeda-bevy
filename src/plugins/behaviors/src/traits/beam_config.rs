use super::SpawnAttack;
use crate::components::{Attacker, BeamCommand, BeamConfig, Target};
use bevy::ecs::system::Commands;

impl SpawnAttack for BeamConfig {
	fn attack(&self, commands: &mut Commands, attacker: Attacker, target: Target) {
		commands
			.entity(attacker.0)
			.insert((*self, BeamCommand { target: target.0 }));
	}
}
