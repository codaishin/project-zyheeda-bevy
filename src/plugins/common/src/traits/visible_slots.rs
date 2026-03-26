use crate::{tools::action_key::slot::SlotKey, traits::accessors::get::View};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct EssenceSlot(pub SlotKey);

impl From<SlotKey> for EssenceSlot {
	fn from(slot_key: SlotKey) -> Self {
		Self(slot_key)
	}
}

impl View<SlotKey> for EssenceSlot {
	fn view(&self) -> SlotKey {
		self.0
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct HandSlot(pub SlotKey);

impl From<SlotKey> for HandSlot {
	fn from(slot_key: SlotKey) -> Self {
		Self(slot_key)
	}
}

impl View<SlotKey> for HandSlot {
	fn view(&self) -> SlotKey {
		self.0
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ForearmSlot(pub SlotKey);

impl From<SlotKey> for ForearmSlot {
	fn from(slot_key: SlotKey) -> Self {
		Self(slot_key)
	}
}

impl View<SlotKey> for ForearmSlot {
	fn view(&self) -> SlotKey {
		self.0
	}
}
