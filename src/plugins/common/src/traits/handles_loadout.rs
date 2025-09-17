pub mod combos_component;
pub mod inventory_component;
pub mod loadout;
pub mod slot_component;

use crate::traits::handles_loadout::{
	combos_component::CombosComponent,
	inventory_component::InventoryComponent,
	loadout::{LoadoutSkill, LoadoutSkillItem, SwapExternal},
	slot_component::SlotComponent,
};

pub trait HandlesLoadout {
	type TItem: LoadoutSkillItem;
	type TSkill: LoadoutSkill;
	type TSkills: IntoIterator<Item = Self::TSkill>;

	type TInventory: InventoryComponent<Self::TItem> + SwapExternal<Self::TSlots>;
	type TSlots: SlotComponent<Self::TItem, Self::TSkills> + SwapExternal<Self::TInventory>;
	type TCombos: CombosComponent<Self::TSkill>;
}
