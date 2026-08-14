use crate::{tools::action_key::slot::SlotKey, traits::accessors::get::View};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct VisibleEssenceSlot(pub SlotKey);

impl From<SlotKey> for VisibleEssenceSlot {
	fn from(slot_key: SlotKey) -> Self {
		Self(slot_key)
	}
}

impl View<SlotKey> for VisibleEssenceSlot {
	fn view(&self) -> SlotKey {
		self.0
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct VisibleHandSlot(pub SlotKey);

impl From<SlotKey> for VisibleHandSlot {
	fn from(slot_key: SlotKey) -> Self {
		Self(slot_key)
	}
}

impl View<SlotKey> for VisibleHandSlot {
	fn view(&self) -> SlotKey {
		self.0
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct VisibleForearmSlot(pub SlotKey);

impl From<SlotKey> for VisibleForearmSlot {
	fn from(slot_key: SlotKey) -> Self {
		Self(slot_key)
	}
}

impl View<SlotKey> for VisibleForearmSlot {
	fn view(&self) -> SlotKey {
		self.0
	}
}
