use common::tools::slot_key::SlotKey;
use std::collections::VecDeque;

pub trait FollowupKeys {
	fn followup_keys<T>(&self, after: T) -> Option<Vec<SlotKey>>
	where
		T: Into<VecDeque<SlotKey>>;
}
