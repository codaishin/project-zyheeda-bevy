use crate::components::blocker_types::BlockerTypes;

pub(crate) trait UpdateBlockers {
	fn update_blockers(&self, _blockers: &mut BlockerTypes) {}
}
