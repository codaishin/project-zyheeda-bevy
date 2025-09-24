use crate::{tools::action_key::slot::SlotKey, traits::accessors::get::GetProperty};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct EssenceSlot(pub SlotKey);

impl From<SlotKey> for EssenceSlot {
	fn from(slot_key: SlotKey) -> Self {
		Self(slot_key)
	}
}

impl GetProperty<SlotKey> for EssenceSlot {
	fn get_property(&self) -> SlotKey {
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

impl GetProperty<SlotKey> for HandSlot {
	fn get_property(&self) -> SlotKey {
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

impl GetProperty<SlotKey> for ForearmSlot {
	fn get_property(&self) -> SlotKey {
		self.0
	}
}
