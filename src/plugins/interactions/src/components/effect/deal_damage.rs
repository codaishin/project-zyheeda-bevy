use super::Effect;
use crate::{traits::act_on::ActOn, InteractionsPlugin};
use bevy::prelude::*;
use common::{
	attributes::health::Health,
	effects::{deal_damage::DealDamage, EffectApplies},
	traits::{
		handles_effect::HandlesEffect,
		handles_life::{ChangeLife, HandlesLife},
	},
};
use std::time::Duration;

impl<TLifecyclePlugin> HandlesEffect<DealDamage> for InteractionsPlugin<TLifecyclePlugin>
where
	TLifecyclePlugin: HandlesLife,
{
	type TTarget = Health;

	fn effect(effect: DealDamage) -> impl Bundle {
		Effect(effect)
	}

	fn attribute(health: Self::TTarget) -> impl Bundle {
		TLifecyclePlugin::TLife::from(health)
	}
}

impl<TLife> ActOn<TLife> for Effect<DealDamage>
where
	TLife: ChangeLife,
{
	fn act(&mut self, _: Entity, life: &mut TLife, delta: Duration) -> EffectApplies {
		let Effect(DealDamage(damage, apply_method)) = *self;

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
		let mut damage = Effect(DealDamage::once(42.));
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
		let mut damage = Effect(DealDamage::once(42.));
		let mut life = Mock_Life::new_mock(|mock| {
			mock.expect_change_by().return_const(());
		});

		let action_type = damage.act(Entity::from_raw(11), &mut life, Duration::from_secs(1));

		assert_eq!(EffectApplies::Once, action_type);
	}

	#[test]
	fn deal_damage_once_per_target() {
		let mut damage = Effect(DealDamage::once_per_target(42.));
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
		let mut damage = Effect(DealDamage::once_per_target(42.));
		let mut life = Mock_Life::new_mock(|mock| {
			mock.expect_change_by().return_const(());
		});

		let action_type = damage.act(Entity::from_raw(11), &mut life, Duration::from_secs(1));

		assert_eq!(EffectApplies::OncePerTarget, action_type);
	}

	#[test]
	fn deal_damage_over_time_scaled_by_delta() {
		let mut damage = Effect(DealDamage::once_per_second(42.));
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
		let mut damage = Effect(DealDamage::once_per_second(42.));
		let mut life = Mock_Life::new_mock(|mock| {
			mock.expect_change_by().return_const(());
		});

		let action_type = damage.act(Entity::from_raw(11), &mut life, Duration::from_secs(1));

		assert_eq!(EffectApplies::Always, action_type);
	}
}
