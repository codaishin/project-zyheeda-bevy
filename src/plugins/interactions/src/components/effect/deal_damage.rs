use super::Effect;
use crate::traits::{act_on::ActOn, is_effect::IsEffect};
use bevy::prelude::Entity;
use common::{
	components::Health,
	effects::{deal_damage::DealDamage, EffectApplies},
};
use std::time::Duration;

impl IsEffect for Effect<DealDamage> {
	type TTarget = Health;
	type TTargetComponent = Health;

	fn attribute(health: Self::TTarget) -> Self::TTargetComponent {
		health
	}
}

impl ActOn<Health> for Effect<DealDamage> {
	fn act(&mut self, _: Entity, health: &mut Health, delta: Duration) -> EffectApplies {
		let Effect(DealDamage(damage, apply_method)) = *self;

		health.current -= match apply_method {
			EffectApplies::Always => damage * delta.as_secs_f32(),
			EffectApplies::Once | EffectApplies::OncePerTarget => damage,
		};

		apply_method
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn deal_damage_once() {
		let mut damage = Effect(DealDamage::once(42.));
		let mut health = Health::new(100.);

		damage.act(
			Entity::from_raw(11),
			&mut health,
			Duration::from_millis(100),
		);

		assert_eq!(
			Health {
				current: 58.,
				max: 100.
			},
			health
		);
	}

	#[test]
	fn action_type_once() {
		let mut damage = Effect(DealDamage::once(42.));
		let mut health = Health::new(100.);

		let action_type = damage.act(Entity::from_raw(11), &mut health, Duration::from_secs(1));

		assert_eq!(EffectApplies::Once, action_type);
	}

	#[test]
	fn deal_damage_once_per_target() {
		let mut damage = Effect(DealDamage::once_per_target(42.));
		let mut health = Health::new(100.);

		damage.act(
			Entity::from_raw(11),
			&mut health,
			Duration::from_millis(100),
		);

		assert_eq!(
			Health {
				current: 58.,
				max: 100.
			},
			health
		);
	}

	#[test]
	fn action_type_once_per_target() {
		let mut damage = Effect(DealDamage::once_per_target(42.));
		let mut health = Health::new(100.);

		let action_type = damage.act(Entity::from_raw(11), &mut health, Duration::from_secs(1));

		assert_eq!(EffectApplies::OncePerTarget, action_type);
	}

	#[test]
	fn deal_damage_over_time_scaled_by_delta() {
		let mut damage = Effect(DealDamage::once_per_second(42.));
		let mut health = Health::new(100.);

		damage.act(
			Entity::from_raw(11),
			&mut health,
			Duration::from_millis(100),
		);

		assert_eq!(
			Health {
				current: 100. - 4.2,
				max: 100.
			},
			health
		);
	}

	#[test]
	fn action_type_always() {
		let mut damage = Effect(DealDamage::once_per_second(42.));
		let mut health = Health::new(100.);

		let action_type = damage.act(Entity::from_raw(11), &mut health, Duration::from_secs(1));

		assert_eq!(EffectApplies::Always, action_type);
	}
}
