use crate::{
	InteractionsPlugin,
	traits::{act_on::ActOn, update_blockers::UpdateBlockers},
};
use bevy::prelude::*;
use common::{
	attributes::health::Health,
	components::{life::Life, persistent_entity::PersistentEntity},
	effects::{EffectApplies, health_damage::HealthDamage},
	traits::handles_effects::HandlesEffect,
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct HealthDamageEffect(pub(crate) HealthDamage);

impl<TSaveGame> HandlesEffect<HealthDamage> for InteractionsPlugin<TSaveGame> {
	type TEffectComponent = HealthDamageEffect;

	fn effect(effect: HealthDamage) -> HealthDamageEffect {
		HealthDamageEffect(effect)
	}

	fn attribute(health: Health) -> impl Bundle {
		Life::from(health)
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
		let Self(HealthDamage(damage, EffectApplies::Always)) = *self else {
			return;
		};

		life.change_by(-damage * delta.as_secs_f32());
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn deal_damage_once() {
		let mut damage = HealthDamageEffect(HealthDamage::once(42.));
		let mut life = Life::from(Health::new(100.));

		damage.on_begin_interaction(PersistentEntity::default(), &mut life);
		damage.on_repeated_interaction(
			PersistentEntity::default(),
			&mut life,
			Duration::from_secs(1),
		);

		let mut expected = Life::from(Health::new(100.));
		expected.change_by(-42.);
		assert_eq!(expected, life);
	}

	#[test]
	fn deal_damage_over_time_scaled_by_delta() {
		let mut damage = HealthDamageEffect(HealthDamage::once_per_second(42.));
		let mut life = Life::from(Health::new(100.));

		damage.on_begin_interaction(PersistentEntity::default(), &mut life);
		damage.on_repeated_interaction(
			PersistentEntity::default(),
			&mut life,
			Duration::from_millis(100),
		);

		let mut expected = Life::from(Health::new(100.));
		expected.change_by(-42. * 0.1);
		assert_eq!(expected, life);
	}
}
