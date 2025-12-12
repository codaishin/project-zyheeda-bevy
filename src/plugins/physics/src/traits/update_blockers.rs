use crate::components::blocker_types::BlockerTypes;

pub(crate) trait UpdateBlockers {
	fn update(&self, _blockers: &mut BlockerTypes) {}
}
