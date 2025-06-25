use crate::{
	InteractionsPlugin,
	components::gravity_affected::{GravityAffected, GravityPull},
	traits::{act_on::ActOn, update_blockers::UpdateBlockers},
};
use bevy::prelude::*;
use common::{
	attributes::affected_by::AffectedBy,
	components::persistent_entity::PersistentEntity,
	effects::gravity::Gravity,
	traits::handles_effect::HandlesEffect,
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct GravityEffect(pub(crate) Gravity);

impl<TDependencies> HandlesEffect<Gravity> for InteractionsPlugin<TDependencies> {
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
	fn on_begin_interaction(&mut self, _: PersistentEntity, _: &mut GravityAffected) {}

	fn on_repeated_interaction(
		&mut self,
		self_entity: PersistentEntity,
		target: &mut GravityAffected,
		_: Duration,
	) {
		let Self(Gravity { strength }) = *self;

		target.push(GravityPull {
			strength,
			towards: self_entity,
		});
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::gravity_affected::GravityPull;
	use common::{tools::UnitsPerSecond, traits::clamp_zero_positive::ClampZeroPositive};

	#[test]
	fn add_gravity_pull() {
		let mut gravity = GravityEffect(Gravity {
			strength: UnitsPerSecond::new(42.),
		});
		let mut gravity_pulls = GravityAffected::default();
		let towards = PersistentEntity::default();

		gravity.on_repeated_interaction(towards, &mut gravity_pulls, Duration::ZERO);

		assert_eq!(
			GravityAffected::new([GravityPull {
				strength: UnitsPerSecond::new(42.),
				towards,
			}]),
			gravity_pulls
		);
	}

	#[test]
	fn no_gravity_pull_on_begin() {
		let mut gravity = GravityEffect(Gravity {
			strength: UnitsPerSecond::new(42.),
		});
		let mut gravity_pulls = GravityAffected::default();

		gravity.on_begin_interaction(PersistentEntity::default(), &mut gravity_pulls);

		assert_eq!(GravityAffected::new([]), gravity_pulls);
	}
}
