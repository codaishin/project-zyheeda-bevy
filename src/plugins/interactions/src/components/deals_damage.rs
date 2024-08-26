use super::ActOn;
use crate::traits::ActionType;
use bevy::prelude::Component;
use common::components::Health;

#[derive(Component, Clone, Debug, PartialEq)]
pub struct DealsDamage(pub i16);

impl ActOn<Health> for DealsDamage {
	fn act_on(&mut self, health: &mut Health) -> ActionType {
		health.current -= self.0;

		ActionType::Once
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn deal_damage() {
		let mut damage = DealsDamage(42);
		let mut health = Health::new(100);

		damage.act_on(&mut health);

		assert_eq!(
			Health {
				current: 58,
				max: 100
			},
			health
		);
	}

	#[test]
	fn action_type_once() {
		let mut damage = DealsDamage(42);
		let mut health = Health::new(100);

		let action_type = damage.act_on(&mut health);

		assert_eq!(ActionType::Once, action_type);
	}
}
