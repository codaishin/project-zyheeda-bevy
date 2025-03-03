use crate::{InteractionsPlugin, traits::act_on::ActOn};
use bevy::prelude::*;
use common::{
	attributes::health::Health,
	effects::{EffectApplies, deal_damage::DealDamage},
	traits::{
		handles_effect::HandlesEffect,
		handles_life::{ChangeLife, HandlesLife},
	},
};
use std::time::Duration;

#[derive(Component, Debug, PartialEq, Clone)]
pub struct DealDamageEffect(pub(crate) DealDamage);

impl<TLifecyclePlugin> HandlesEffect<DealDamage> for InteractionsPlugin<TLifecyclePlugin>
where
	TLifecyclePlugin: HandlesLife,
{
	type TTarget = Health;
	type TEffectComponent = DealDamageEffect;

	fn effect(effect: DealDamage) -> Self::TEffectComponent {
		DealDamageEffect(effect)
	}

	fn attribute(health: Self::TTarget) -> impl Bundle {
		TLifecyclePlugin::TLife::from(health)
	}
}

impl<TLife> ActOn<TLife> for DealDamageEffect
where
	TLife: ChangeLife,
{
	fn act(&mut self, _: Entity, life: &mut TLife, delta: Duration) -> EffectApplies {
		let Self(DealDamage(damage, apply_method)) = *self;

		let change = match apply_method {
			EffectApplies::Always => -damage * delta.as_secs_f32(),
			EffectApplies::Once | EffectApplies::OncePerTarget => -damage,
		};
		life.change_by(change);

		apply_method
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

		damage.act(Entity::from_raw(11), &mut life, Duration::from_millis(100));
	}

	#[test]
	fn action_type_once() {
		let mut damage = DealDamageEffect(DealDamage::once(42.));
		let mut life = Mock_Life::new_mock(|mock| {
			mock.expect_change_by().return_const(());
		});

		let action_type = damage.act(Entity::from_raw(11), &mut life, Duration::from_secs(1));

		assert_eq!(EffectApplies::Once, action_type);
	}

	#[test]
	fn deal_damage_once_per_target() {
		let mut damage = DealDamageEffect(DealDamage::once_per_target(42.));
		let mut life = Mock_Life::new_mock(|mock| {
			mock.expect_change_by()
				.times(1)
				.with(eq(-42.))
				.return_const(());
		});

		damage.act(Entity::from_raw(11), &mut life, Duration::from_millis(100));
	}

	#[test]
	fn action_type_once_per_target() {
		let mut damage = DealDamageEffect(DealDamage::once_per_target(42.));
		let mut life = Mock_Life::new_mock(|mock| {
			mock.expect_change_by().return_const(());
		});

		let action_type = damage.act(Entity::from_raw(11), &mut life, Duration::from_secs(1));

		assert_eq!(EffectApplies::OncePerTarget, action_type);
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

		damage.act(Entity::from_raw(11), &mut life, Duration::from_millis(100));
	}

	#[test]
	fn action_type_always() {
		let mut damage = DealDamageEffect(DealDamage::once_per_second(42.));
		let mut life = Mock_Life::new_mock(|mock| {
			mock.expect_change_by().return_const(());
		});

		let action_type = damage.act(Entity::from_raw(11), &mut life, Duration::from_secs(1));

		assert_eq!(EffectApplies::Always, action_type);
	}
}
