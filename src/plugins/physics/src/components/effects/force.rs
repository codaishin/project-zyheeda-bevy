use crate::{
	ActOn,
	PhysicsPlugin,
	components::{affected::force_affected::ForceAffected, blocker_types::BlockerTypes},
	traits::update_blockers::UpdateBlockers,
};
use bevy::prelude::*;
use common::prelude::*;
use macros::{EntityKey, SavableComponent, serde_model};
use std::time::Duration;

#[serde_model]
#[derive(Component, SavableComponent, Debug, PartialEq, Clone)]
#[savable_component(id = "force_effect")]
pub struct ForceEffect(pub(crate) Force);

#[derive(EntityKey)]
pub struct ForceEntity {
	entity: Entity,
}

impl<TDependencies> HandlesPhysicalEffect<Force> for PhysicsPlugin<TDependencies> {
	type TEffectAdded = Added<ForceEffect>;
	type TAffectedEntity = ForceEntity;
	type TAffected = Query<'static, 'static, Ref<'static, ForceAffected>>;
}

impl UpdateBlockers for ForceEffect {
	fn update_blockers(&self, BlockerTypes(blockers): &mut BlockerTypes) {
		blockers.insert(Blocker::Force);
	}
}

impl ActOn<ForceAffected> for ForceEffect {
	fn on_begin_interaction(&mut self, _: PersistentEntity, _: &mut ForceAffected) {}

	fn on_repeated_interaction(&mut self, _: PersistentEntity, _: &mut ForceAffected, _: Duration) {
		// FIXME: Target should be moved outside the force effect collider
	}
}
