use common::components::is_blocker::IsBlocker;

pub(crate) trait UpdateBlockers {
	fn update(&self, _blockers: &mut IsBlocker) {}
}
