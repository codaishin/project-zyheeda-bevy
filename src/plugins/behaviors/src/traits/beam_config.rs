use super::SpawnAttack;
use crate::components::{Attacker, BeamCommand, BeamConfig, Target};
use bevy::ecs::system::Commands;

impl SpawnAttack for BeamConfig {
	fn attack(&self, commands: &mut Commands, attacker: Attacker, target: Target) {
		commands.spawn((
			*self,
			BeamCommand {
				source: attacker.0,
				target: target.0,
			},
		));
	}
}
