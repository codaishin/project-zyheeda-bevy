use crate::{
	InteractionsPlugin,
	components::gravity_affected::{GravityAffected, GravityPull},
	traits::{act_on::ActOn, update_blockers::UpdateBlockers},
};
use bevy::prelude::*;
use common::{
	attributes::affected_by::AffectedBy,
	effects::{EffectApplies, gravity::Gravity},
	traits::handles_effect::HandlesEffect,
};
use std::time::Duration;

#[derive(Component, Debug, PartialEq, Clone)]
pub struct GravityEffect(pub(crate) Gravity);

impl<TLifecyclePlugin> HandlesEffect<Gravity> for InteractionsPlugin<TLifecyclePlugin> {
	type TTarget = AffectedBy<Gravity>;
	type TEffectComponent = GravityEffect;

	fn effect(effect: Gravity) -> Self::TEffectComponent {
		GravityEffect(effect)
	}

	fn attribute(_: Self::TTarget) -> impl Bundle {
		GravityAffected::default()
	}
}

impl UpdateBlockers for GravityEffect {}

impl ActOn<GravityAffected> for GravityEffect {
	fn act(
		&mut self,
		self_entity: Entity,
		target: &mut GravityAffected,
		_: Duration,
	) -> EffectApplies {
		let Self(gravity) = self;

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
		let mut gravity = GravityEffect(Gravity {
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
		let mut gravity = GravityEffect(Gravity {
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
