use crate::{
	components::void_beam::VoidBeamAttack,
	traits::{GetAttackSpawner, SpawnAttack},
};
use bevy::prelude::*;
use common::traits::{
	accessors::get::GetterRef,
	handles_enemies::{AttackConfig, AttackMethod},
};
use std::{sync::Arc, time::Duration};

pub(crate) struct AttackSpawnerFactory;

impl<TEnemy> GetAttackSpawner<TEnemy> for AttackSpawnerFactory
where
	TEnemy: GetterRef<AttackConfig>,
{
	fn attack_spawner(enemy: &TEnemy) -> Arc<dyn SpawnAttack> {
		let attack = enemy.get();
		match attack.method {
			AttackMethod::VoidBeam => Arc::new(VoidBeamAttack {
				damage: 10.,
				color: Color::BLACK,
				emissive: LinearRgba::new(23.0, 23.0, 23.0, 1.),
				lifetime: Duration::from_secs(1),
				range: attack.range,
			}),
		}
	}
}
