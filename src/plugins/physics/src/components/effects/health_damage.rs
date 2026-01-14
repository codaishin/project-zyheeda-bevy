use crate::{
	PhysicsPlugin,
	components::affected::life::Life,
	traits::{act_on::ActOn, update_blockers::UpdateBlockers},
};
use bevy::prelude::*;
use common::{
	components::persistent_entity::PersistentEntity,
	effects::{EffectApplies, health_damage::HealthDamage},
	traits::handles_physics::HandlesPhysicalEffect,
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct HealthDamageEffect(pub(crate) HealthDamage);

impl<TSaveGame> HandlesPhysicalEffect<HealthDamage> for PhysicsPlugin<TSaveGame> {
	type TEffectComponent = HealthDamageEffect;
	type TAffectedComponent = Life;

	fn into_effect_component(effect: HealthDamage) -> HealthDamageEffect {
		HealthDamageEffect(effect)
	}
}

impl UpdateBlockers for HealthDamageEffect {}

impl ActOn<Life> for HealthDamageEffect {
	fn on_begin_interaction(&mut self, _: PersistentEntity, life: &mut Life) {
		let Self(HealthDamage(damage, EffectApplies::Once)) = *self else {
			return;
		};

		life.change_by(-damage);
	}

	fn on_repeated_interaction(&mut self, _: PersistentEntity, life: &mut Life, delta: Duration) {
		let Self(HealthDamage(damage, EffectApplies::OncePerSecond)) = *self else {
			return;
		};

		life.change_by(-damage * delta.as_secs_f32());
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::attributes::health::Health;

	#[test]
	fn deal_damage_once() {
		let mut damage = HealthDamageEffect(HealthDamage::once(42.));
		let mut life = Life(Health::new(100.));

		damage.on_begin_interaction(PersistentEntity::default(), &mut life);
		damage.on_repeated_interaction(
			PersistentEntity::default(),
			&mut life,
			Duration::from_secs(1),
		);

		let mut expected = Life(Health::new(100.));
		expected.change_by(-42.);
		assert_eq!(expected, life);
	}

	#[test]
	fn deal_damage_over_time_scaled_by_delta() {
		let mut damage = HealthDamageEffect(HealthDamage::per_second(42.));
		let mut life = Life(Health::new(100.));

		damage.on_begin_interaction(PersistentEntity::default(), &mut life);
		damage.on_repeated_interaction(
			PersistentEntity::default(),
			&mut life,
			Duration::from_millis(100),
		);

		let mut expected = Life(Health::new(100.));
		expected.change_by(-42. * 0.1);
		assert_eq!(expected, life);
	}
}
