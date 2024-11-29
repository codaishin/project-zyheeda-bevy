use super::effected_by_gravity::{EffectedByGravity, Pull};
use crate::traits::ActOn;
use bevy::prelude::{Component, Entity};
use common::{effects::EffectApplies, tools::UnitsPerSecond};
use std::time::Duration;

#[derive(Component, Debug, PartialEq, Clone)]
pub struct Gravity {
	pub strength: UnitsPerSecond,
}

impl ActOn<EffectedByGravity> for Gravity {
	fn act(
		&mut self,
		self_entity: Entity,
		target: &mut EffectedByGravity,
		_: Duration,
	) -> EffectApplies {
		target.pulls.push(Pull {
			strength: self.strength,
			towards: self_entity,
		});
		EffectApplies::Always
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{effected_by::EffectedBy, effected_by_gravity::Pull};
	use common::traits::clamp_zero_positive::ClampZeroPositive;

	#[test]
	fn action_type_always() {
		let mut gravity = Gravity {
			strength: UnitsPerSecond::new(42.),
		};

		let action_type = gravity.act(
			Entity::from_raw(42),
			&mut EffectedBy::gravity(),
			Duration::ZERO,
		);

		assert_eq!(action_type, EffectApplies::Always);
	}

	#[test]
	fn add_gravity_pull() {
		let mut gravity = Gravity {
			strength: UnitsPerSecond::new(42.),
		};
		let mut effected_by_gravity = EffectedBy::gravity();

		gravity.act(
			Entity::from_raw(42),
			&mut effected_by_gravity,
			Duration::ZERO,
		);

		assert_eq!(
			EffectedByGravity {
				pulls: vec![Pull {
					strength: UnitsPerSecond::new(42.),
					towards: Entity::from_raw(42),
				}]
			},
			effected_by_gravity
		);
	}
}
