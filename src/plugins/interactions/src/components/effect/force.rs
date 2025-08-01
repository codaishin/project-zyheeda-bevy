use crate::{
	ActOn,
	InteractionsPlugin,
	components::force_affected::ForceAffected,
	traits::update_blockers::UpdateBlockers,
};
use bevy::prelude::*;
use common::{
	attributes::affected_by::AffectedBy,
	components::{
		is_blocker::{Blocker, IsBlocker},
		persistent_entity::PersistentEntity,
	},
	effects::force::Force,
	traits::handles_effect::HandlesEffect,
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ForceEffect(pub(crate) Force);

impl<TDependencies> HandlesEffect<Force> for InteractionsPlugin<TDependencies> {
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
	fn update_blockers(&self, IsBlocker(blockers): &mut IsBlocker) {
		blockers.insert(Blocker::Force);
	}
}

impl ActOn<ForceAffected> for ForceEffect {
	fn on_begin_interaction(&mut self, _: PersistentEntity, _: &mut ForceAffected) {}

	fn on_repeated_interaction(&mut self, _: PersistentEntity, _: &mut ForceAffected, _: Duration) {
		// FIXME: Target should be moved outside the force effect collider
	}
}
