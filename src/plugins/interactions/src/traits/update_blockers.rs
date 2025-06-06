use common::blocker::Blockers;

pub(crate) trait UpdateBlockers {
	fn update_blockers(&self, _blockers: &mut Blockers) {}
}
