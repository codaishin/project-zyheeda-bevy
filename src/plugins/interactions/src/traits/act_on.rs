use crate::traits::update_blockers::UpdateBlockers;
use bevy::prelude::*;
use common::effects::EffectApplies;
use std::time::Duration;

pub(crate) trait ActOn<TTarget>: UpdateBlockers {
	fn act(&mut self, self_entity: Entity, target: &mut TTarget, delta: Duration) -> EffectApplies;
}
