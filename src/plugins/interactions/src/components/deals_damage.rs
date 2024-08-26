use super::ActOn;
use crate::traits::ActionType;
use bevy::prelude::Component;
use common::components::Health;
use std::time::Duration;

#[derive(Component, Clone, Debug, PartialEq)]
pub struct DealsDamage(f32, ActionType);

impl DealsDamage {
	pub fn once(amount: f32) -> Self {
		DealsDamage(amount, ActionType::Once)
	}

	pub fn once_per_target(amount: f32) -> Self {
		DealsDamage(amount, ActionType::OncePerTarget)
	}

	pub fn once_per_second(amount: f32) -> Self {
		DealsDamage(amount, ActionType::Always)
	}
}

impl ActOn<Health> for DealsDamage {
	fn act_on(&mut self, health: &mut Health, delta: Duration) -> ActionType {
		let DealsDamage(damage, action_type) = *self;

		health.current -= match action_type {
			ActionType::Always => damage * delta.as_secs_f32(),
			ActionType::Once | ActionType::OncePerTarget => damage,
		};

		action_type
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn deal_damage_once() {
		let mut damage = DealsDamage::once(42.);
		let mut health = Health::new(100.);

		damage.act_on(&mut health, Duration::from_millis(100));

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
		let mut damage = DealsDamage::once(42.);
		let mut health = Health::new(100.);

		let action_type = damage.act_on(&mut health, Duration::from_secs(1));

		assert_eq!(ActionType::Once, action_type);
	}

	#[test]
	fn deal_damage_once_per_target() {
		let mut damage = DealsDamage::once_per_target(42.);
		let mut health = Health::new(100.);

		damage.act_on(&mut health, Duration::from_millis(100));

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
		let mut damage = DealsDamage::once_per_target(42.);
		let mut health = Health::new(100.);

		let action_type = damage.act_on(&mut health, Duration::from_secs(1));

		assert_eq!(ActionType::OncePerTarget, action_type);
	}

	#[test]
	fn deal_damage_over_time_scaled_by_delta() {
		let mut damage = DealsDamage::once_per_second(42.);
		let mut health = Health::new(100.);

		damage.act_on(&mut health, Duration::from_millis(100));

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
		let mut damage = DealsDamage::once_per_second(42.);
		let mut health = Health::new(100.);

		let action_type = damage.act_on(&mut health, Duration::from_secs(1));

		assert_eq!(ActionType::Always, action_type);
	}
}
