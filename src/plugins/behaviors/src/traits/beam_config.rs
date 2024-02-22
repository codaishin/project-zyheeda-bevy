use super::{DespawnFn, SpawnAttack};
use crate::components::{Attacker, BeamCommand, BeamConfig, Target};
use bevy::{
	ecs::{entity::Entity, system::Commands},
	hierarchy::DespawnRecursiveExt,
};
use std::sync::Arc;

impl SpawnAttack for BeamConfig {
	fn spawn(&self, commands: &mut Commands, attacker: Attacker, target: Target) -> DespawnFn {
		let beam = commands
			.spawn((
				*self,
				BeamCommand {
					source: attacker.0,
					target: target.0,
				},
			))
			.id();

		despawn(beam)
	}
}

fn despawn(entity: Entity) -> DespawnFn {
	Arc::new(move |commands| {
		let Some(entity) = commands.get_entity(entity) else {
			return;
		};
		entity.despawn_recursive();
	})
}
