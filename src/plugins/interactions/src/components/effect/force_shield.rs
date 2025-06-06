use crate::{ActOn, InteractionsPlugin, components::force_affected::ForceAffected};
use bevy::prelude::*;
use common::{
	attributes::affected_by::AffectedBy,
	effects::{EffectApplies, force_shield::ForceShield},
	traits::handles_effect::HandlesEffect,
};
use std::time::Duration;

#[derive(Component, Debug, PartialEq)]
pub struct ForceShieldEffect(pub(crate) ForceShield);

impl<TLifecyclePlugin> HandlesEffect<ForceShield> for InteractionsPlugin<TLifecyclePlugin> {
	type TTarget = AffectedBy<ForceShield>;
	type TEffectComponent = ForceShieldEffect;

	fn effect(effect: ForceShield) -> Self::TEffectComponent {
		ForceShieldEffect(effect)
	}

	fn attribute(_: Self::TTarget) -> impl Bundle {}
}

impl ActOn<ForceAffected> for ForceShield {
	fn act(&mut self, _: Entity, _: &mut ForceAffected, _: Duration) -> EffectApplies {
		// FIXME: Target should be moved outside the force shield on context via some kind of force
		EffectApplies::Always
	}
}
