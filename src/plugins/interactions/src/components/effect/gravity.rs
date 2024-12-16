use super::Effect;
use crate::{
	components::gravity_affected::{GravityAffected, GravityPull},
	traits::act_on::ActOn,
	InteractionsPlugin,
};
use bevy::prelude::*;
use common::{
	attributes::affected_by::AffectedBy,
	effects::{gravity::Gravity, EffectApplies},
	traits::handles_effect::HandlesEffect,
};
use std::time::Duration;

impl<TPrefabs, TLifecyclePlugin> HandlesEffect<Gravity>
	for InteractionsPlugin<TPrefabs, TLifecyclePlugin>
{
	type TTarget = AffectedBy<Gravity>;
	type TEffectComponent = Effect<Gravity>;

	fn effect(effect: Gravity) -> Self::TEffectComponent {
		Effect(effect)
	}

	fn attribute(_: Self::TTarget) -> impl Bundle {
		GravityAffected::default()
	}
}

impl ActOn<GravityAffected> for Effect<Gravity> {
	fn act(
		&mut self,
		self_entity: Entity,
		target: &mut GravityAffected,
		_: Duration,
	) -> EffectApplies {
		let Effect(gravity) = self;

		target.push(GravityPull {
			strength: gravity.strength,
			towards: self_entity,
		});

		gravity.effect_applies
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::gravity_affected::GravityPull;
	use common::{tools::UnitsPerSecond, traits::clamp_zero_positive::ClampZeroPositive};

	#[test]
	fn proper_action_type() {
		let mut gravity = Effect(Gravity {
			strength: UnitsPerSecond::new(42.),
			effect_applies: EffectApplies::Once,
		});

		let action_type = gravity.act(
			Entity::from_raw(42),
			&mut GravityAffected::default(),
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
		let mut gravity_pulls = GravityAffected::default();

		gravity.act(Entity::from_raw(42), &mut gravity_pulls, Duration::ZERO);

		assert_eq!(
			GravityAffected::new([GravityPull {
				strength: UnitsPerSecond::new(42.),
				towards: Entity::from_raw(42),
			}]),
			gravity_pulls
		);
	}
}
