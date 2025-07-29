use common::components::is_blocker::IsBlocker;

pub(crate) trait UpdateBlockers {
	fn update_blockers(&self, _blockers: &mut IsBlocker) {}
}
