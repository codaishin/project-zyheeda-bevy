use crate::{
	InteractionsPlugin,
	traits::{act_on::ActOn, update_blockers::UpdateBlockers},
};
use bevy::prelude::*;
use common::{
	attributes::health::Health,
	components::{life::Life, persistent_entity::PersistentEntity},
	effects::{EffectApplies, deal_damage::DealDamage},
	traits::handles_effect::HandlesEffect,
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct DealDamageEffect(pub(crate) DealDamage);

impl<TSaveGame> HandlesEffect<DealDamage> for InteractionsPlugin<TSaveGame> {
	type TTarget = Health;
	type TEffectComponent = DealDamageEffect;

	fn effect(effect: DealDamage) -> DealDamageEffect {
		DealDamageEffect(effect)
	}

	fn attribute(health: Health) -> impl Bundle {
		Life::from(health)
	}
}

impl UpdateBlockers for DealDamageEffect {}

impl ActOn<Life> for DealDamageEffect {
	fn on_begin_interaction(&mut self, _: PersistentEntity, life: &mut Life) {
		let Self(DealDamage(damage, EffectApplies::Once)) = *self else {
			return;
		};

		life.change_by(-damage);
	}

	fn on_repeated_interaction(&mut self, _: PersistentEntity, life: &mut Life, delta: Duration) {
		let Self(DealDamage(damage, EffectApplies::Always)) = *self else {
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
		let mut damage = DealDamageEffect(DealDamage::once(42.));
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
		let mut damage = DealDamageEffect(DealDamage::once_per_second(42.));
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
