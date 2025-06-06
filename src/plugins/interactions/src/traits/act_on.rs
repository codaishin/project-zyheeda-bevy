use crate::traits::update_blockers::UpdateBlockers;
use bevy::prelude::*;
use std::time::Duration;

pub(crate) trait ActOn<TTarget>: UpdateBlockers {
	fn on_begin_interaction(&mut self, self_entity: Entity, target: &mut TTarget);
	fn on_repeated_interaction(
		&mut self,
		self_entity: Entity,
		target: &mut TTarget,
		delta: Duration,
	);
}
