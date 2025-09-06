pub mod combos_component;
pub mod inventory_component;
pub mod loadout;
pub mod slot_component;

use crate::traits::handles_loadout::{
	combos_component::CombosComponent,
	inventory_component::InventoryComponent,
	loadout::{LoadoutItemEntry, LoadoutSkill, SwapExternal},
	slot_component::SlotComponent,
};

pub trait HandlesLoadout {
	type TItemEntry: LoadoutItemEntry;
	type TSkill: LoadoutSkill;
	type TSkills: IntoIterator<Item = Self::TSkill>;

	type TInventory: InventoryComponent<Self::TItemEntry> + SwapExternal<Self::TSlots>;
	type TSlots: SlotComponent<Self::TItemEntry, Self::TSkills> + SwapExternal<Self::TInventory>;
	type TCombos: CombosComponent<Self::TSkill>;
}
