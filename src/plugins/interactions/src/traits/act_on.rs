use crate::traits::update_blockers::UpdateBlockers;
use common::components::persistent_entity::PersistentEntity;
use std::time::Duration;

pub(crate) trait ActOn<TTarget>: UpdateBlockers {
	fn on_begin_interaction(&mut self, self_entity: PersistentEntity, target: &mut TTarget);
	fn on_repeated_interaction(
		&mut self,
		self_entity: PersistentEntity,
		target: &mut TTarget,
		delta: Duration,
	);
}
