use common::tools::keys::slot::SlotKey;
use std::collections::VecDeque;

pub trait FollowupKeys {
	fn followup_keys<T>(&self, after: T) -> Option<Vec<SlotKey>>
	where
		T: Into<VecDeque<SlotKey>>;
}
