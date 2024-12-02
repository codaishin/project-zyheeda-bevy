use bevy::prelude::*;
use common::effects::EffectApplies;
use std::time::Duration;

pub(crate) trait ActOn<TTarget> {
	fn act(&mut self, self_entity: Entity, target: &mut TTarget, delta: Duration) -> EffectApplies;
}
