use super::{inventory_key::InventoryKey, keys::slot::SlotKey};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum SwapKey {
	Inventory(InventoryKey),
	Slot(SlotKey),
}

impl From<InventoryKey> for SwapKey {
	fn from(key: InventoryKey) -> Self {
		Self::Inventory(key)
	}
}

impl From<SlotKey> for SwapKey {
	fn from(key: SlotKey) -> Self {
		Self::Slot(key)
	}
}
