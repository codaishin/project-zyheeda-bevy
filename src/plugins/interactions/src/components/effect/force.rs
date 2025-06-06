use crate::{
	ActOn,
	InteractionsPlugin,
	components::force_affected::ForceAffected,
	traits::update_blockers::UpdateBlockers,
};
use bevy::prelude::*;
use common::{
	attributes::affected_by::AffectedBy,
	blocker::{Blocker, Blockers},
	effects::{EffectApplies, force_shield::Force},
	traits::handles_effect::HandlesEffect,
};
use std::time::Duration;

#[derive(Component, Debug, PartialEq, Clone)]
pub struct ForceEffect(pub(crate) Force);

impl<TLifecyclePlugin> HandlesEffect<Force> for InteractionsPlugin<TLifecyclePlugin> {
	type TTarget = AffectedBy<Force>;
	type TEffectComponent = ForceEffect;

	fn effect(effect: Force) -> Self::TEffectComponent {
		ForceEffect(effect)
	}

	fn attribute(_: Self::TTarget) -> impl Bundle {
		ForceAffected
	}
}

impl UpdateBlockers for ForceEffect {
	fn update_blockers(&self, Blockers(blockers): &mut Blockers) {
		blockers.insert(Blocker::Force);
	}
}

impl ActOn<ForceAffected> for ForceEffect {
	fn act(&mut self, _: Entity, _: &mut ForceAffected, _: Duration) -> EffectApplies {
		// FIXME: Target should be moved outside the force shield on context via some kind of force
		EffectApplies::Always
	}
}
