use crate::tools::action_key::slot::SlotKey;
use bevy::prelude::*;

pub trait VisibleSlots: Component {
	fn visualize_keys(&self) -> impl Iterator<Item = SlotKey>;
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct EssenceSlot(pub SlotKey);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct HandSlot(pub SlotKey);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ForearmSlot(pub SlotKey);
