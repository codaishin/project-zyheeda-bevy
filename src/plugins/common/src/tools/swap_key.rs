use crate::tools::action_key::slot::SlotKey;

use super::inventory_key::InventoryKey;

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

impl<T> From<T> for SwapKey
where
	T: Into<SlotKey>,
{
	fn from(key: T) -> Self {
		Self::Slot(key.into())
	}
}
