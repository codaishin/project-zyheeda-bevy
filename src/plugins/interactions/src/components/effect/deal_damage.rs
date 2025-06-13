use crate::{
	InteractionsPlugin,
	traits::{act_on::ActOn, update_blockers::UpdateBlockers},
};
use bevy::prelude::*;
use common::{
	attributes::health::Health,
	components::persistent_entity::PersistentEntity,
	effects::{EffectApplies, deal_damage::DealDamage},
	traits::{
		handles_effect::HandlesEffect,
		handles_life::{ChangeLife, HandlesLife},
	},
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Component, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct DealDamageEffect(pub(crate) DealDamage);

impl<TSaveGame, TLifeCycle> HandlesEffect<DealDamage>
	for InteractionsPlugin<(TSaveGame, TLifeCycle)>
where
	TLifeCycle: HandlesLife,
{
	type TTarget = Health;
	type TEffectComponent = DealDamageEffect;

	fn effect(effect: DealDamage) -> Self::TEffectComponent {
		DealDamageEffect(effect)
	}

	fn attribute(health: Self::TTarget) -> impl Bundle {
		TLifeCycle::TLife::from(health)
	}
}

impl UpdateBlockers for DealDamageEffect {}

impl<TLife> ActOn<TLife> for DealDamageEffect
where
	TLife: ChangeLife,
{
	fn on_begin_interaction(&mut self, _: PersistentEntity, life: &mut TLife) {
		let Self(DealDamage(damage, EffectApplies::Once)) = *self else {
			return;
		};

		life.change_by(-damage);
	}

	fn on_repeated_interaction(&mut self, _: PersistentEntity, life: &mut TLife, delta: Duration) {
		let Self(DealDamage(damage, EffectApplies::Always)) = *self else {
			return;
		};

		life.change_by(-damage * delta.as_secs_f32());
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{simple_init, traits::mock::Mock};
	use mockall::{mock, predicate::eq};

	mock! {
		_Life {}
		impl ChangeLife for _Life {
			fn change_by(&mut self, value: f32);
		}
	}

	simple_init!(Mock_Life);

	#[test]
	fn deal_damage_once() {
		let mut damage = DealDamageEffect(DealDamage::once(42.));
		let mut life = Mock_Life::new_mock(|mock| {
			mock.expect_change_by()
				.times(1)
				.with(eq(-42.))
				.return_const(());
		});

		damage.on_begin_interaction(PersistentEntity::default(), &mut life);
		damage.on_repeated_interaction(
			PersistentEntity::default(),
			&mut life,
			Duration::from_millis(100),
		);
	}

	#[test]
	fn deal_damage_over_time_scaled_by_delta() {
		let mut damage = DealDamageEffect(DealDamage::once_per_second(42.));
		let mut life = Mock_Life::new_mock(|mock| {
			mock.expect_change_by()
				.times(1)
				.with(eq(-42. * 0.1))
				.return_const(());
		});

		damage.on_begin_interaction(PersistentEntity::default(), &mut life);
		damage.on_repeated_interaction(
			PersistentEntity::default(),
			&mut life,
			Duration::from_millis(100),
		);
	}
}
