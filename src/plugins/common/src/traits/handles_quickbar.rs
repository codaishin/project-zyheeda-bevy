use crate::tools::slot_key::SlotKey;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ActiveSlotKey(pub SlotKey);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct QueuedSlotKey(pub SlotKey);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ComboSlotKey(pub SlotKey);
