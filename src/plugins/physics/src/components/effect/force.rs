use crate::{
	ActOn,
	PhysicsPlugin,
	components::affected::force_affected::ForceAffected,
	traits::update_blockers::UpdateBlockers,
};
use bevy::prelude::*;
use common::{
	components::{
		is_blocker::{Blocker, IsBlocker},
		persistent_entity::PersistentEntity,
	},
	effects::force::Force,
	traits::handles_physics::HandlesPhysicalEffect,
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ForceEffect(pub(crate) Force);

impl<TDependencies> HandlesPhysicalEffect<Force> for PhysicsPlugin<TDependencies> {
	type TEffectComponent = ForceEffect;
	type TAffectedComponent = ForceAffected;

	fn into_effect_component(effect: Force) -> ForceEffect {
		ForceEffect(effect)
	}
}

impl UpdateBlockers for ForceEffect {
	fn update(&self, IsBlocker(blockers): &mut IsBlocker) {
		blockers.insert(Blocker::Force);
	}
}

impl ActOn<ForceAffected> for ForceEffect {
	fn on_begin_interaction(&mut self, _: PersistentEntity, _: &mut ForceAffected) {}

	fn on_repeated_interaction(&mut self, _: PersistentEntity, _: &mut ForceAffected, _: Duration) {
		// FIXME: Target should be moved outside the force effect collider
	}
}
