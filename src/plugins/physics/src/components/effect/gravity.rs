use crate::{
	PhysicsPlugin,
	components::gravity_affected::{GravityAffected, GravityPull},
	traits::{act_on::ActOn, update_blockers::UpdateBlockers},
};
use bevy::prelude::*;
use common::{
	attributes::effect_target::EffectTarget,
	components::persistent_entity::PersistentEntity,
	effects::gravity::Gravity,
	traits::handles_physics::HandlesPhysicalEffect,
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct GravityEffect(pub(crate) Gravity);

impl<TDependencies> HandlesPhysicalEffect<Gravity> for PhysicsPlugin<TDependencies> {
	type TEffectComponent = GravityEffect;
	type TAffectedComponent = GravityAffected;

	fn into_effect_component(effect: Gravity) -> GravityEffect {
		GravityEffect(effect)
	}

	fn into_affected_component(effect_target: EffectTarget<Gravity>) -> GravityAffected {
		match effect_target {
			EffectTarget::Affected => GravityAffected::AffectedBy { pulls: vec![] },
			EffectTarget::Immune => GravityAffected::Immune,
		}
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
	use common::tools::UnitsPerSecond;

	#[test]
	fn add_gravity_pull() {
		let mut gravity = GravityEffect(Gravity {
			strength: UnitsPerSecond::from(42.),
		});
		let mut gravity_pulls = GravityAffected::affected([]);
		let towards = PersistentEntity::default();

		gravity.on_repeated_interaction(towards, &mut gravity_pulls, Duration::ZERO);

		assert_eq!(
			GravityAffected::affected([GravityPull {
				strength: UnitsPerSecond::from(42.),
				towards,
			}]),
			gravity_pulls
		);
	}

	#[test]
	fn no_gravity_pull_on_begin() {
		let mut gravity = GravityEffect(Gravity {
			strength: UnitsPerSecond::from(42.),
		});
		let mut gravity_pulls = GravityAffected::affected([]);

		gravity.on_begin_interaction(PersistentEntity::default(), &mut gravity_pulls);

		assert_eq!(GravityAffected::affected([]), gravity_pulls);
	}
}
