use crate::tools::action_key::slot::SlotKey;
use bevy::prelude::*;

pub trait VisibleSlots: Component {
	fn visualize_keys(&self) -> impl Iterator<Item = SlotKey>;
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct EssenceSlot(pub SlotKey);

impl From<&EssenceSlot> for SlotKey {
	fn from(EssenceSlot(slot_key): &EssenceSlot) -> Self {
		*slot_key
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct HandSlot(pub SlotKey);

impl From<&HandSlot> for SlotKey {
	fn from(HandSlot(slot_key): &HandSlot) -> Self {
		*slot_key
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ForearmSlot(pub SlotKey);

impl From<&ForearmSlot> for SlotKey {
	fn from(ForearmSlot(slot_key): &ForearmSlot) -> Self {
		*slot_key
	}
}
