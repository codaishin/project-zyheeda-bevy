use super::Effect;
use crate::{
	components::effected_by_gravity::{EffectedByGravity, Pull},
	traits::ActOn,
};
use bevy::prelude::*;
use common::effects::{gravity::Gravity, EffectApplies};
use std::time::Duration;

impl ActOn<EffectedByGravity> for Effect<Gravity> {
	fn act(
		&mut self,
		self_entity: Entity,
		target: &mut EffectedByGravity,
		_: Duration,
	) -> EffectApplies {
		let Effect(gravity) = self;

		target.pulls.push(Pull {
			strength: gravity.strength,
			towards: self_entity,
		});

		gravity.effect_applies
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{effected_by::EffectedBy, effected_by_gravity::Pull};
	use common::{tools::UnitsPerSecond, traits::clamp_zero_positive::ClampZeroPositive};

	#[test]
	fn proper_action_type() {
		let mut gravity = Effect(Gravity {
			strength: UnitsPerSecond::new(42.),
			effect_applies: EffectApplies::Once,
		});

		let action_type = gravity.act(
			Entity::from_raw(42),
			&mut EffectedBy::gravity(),
			Duration::ZERO,
		);

		assert_eq!(action_type, EffectApplies::Once);
	}

	#[test]
	fn add_gravity_pull() {
		let mut gravity = Effect(Gravity {
			strength: UnitsPerSecond::new(42.),
			..default()
		});
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
