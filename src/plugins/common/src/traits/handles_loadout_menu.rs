use super::{inspect_able::InspectAble, thread_safe::ThreadSafe};
use crate::tools::{
	inventory_key::InventoryKey,
	item_description::ItemDescription,
	skill_execution::SkillExecution,
	skill_icon::SkillIcon,
	slot_key::SlotKey,
	swap_key::SwapKey,
};
use bevy::prelude::*;

pub trait HandlesLoadoutMenu {
	fn loadout_with_swapper<TSwap>() -> impl ConfigureInventory<TSwap>
	where
		TSwap: Component + SwapValuesByKey;

	fn configure_quickbar_menu<TContainer, TSystemMarker>(
		app: &mut App,
		get_quickbar_cache: impl IntoSystem<(), Option<TContainer>, TSystemMarker>,
	) where
		TContainer: GetItem<SlotKey> + ThreadSafe,
		TContainer::TItem:
			InspectAble<ItemDescription> + InspectAble<SkillIcon> + InspectAble<SkillExecution>;
}

pub trait ConfigureInventory<TSwap> {
	fn configure<TInventory, TSlots, TSystemMarker1, TSystemMarker2>(
		&self,
		app: &mut App,
		get_inventor_descriptors: impl IntoSystem<(), Option<TInventory>, TSystemMarker1>,
		get_slot_descriptors: impl IntoSystem<(), Option<TSlots>, TSystemMarker2>,
	) where
		TInventory: GetItem<InventoryKey> + ThreadSafe,
		TInventory::TItem: InspectAble<ItemDescription>,
		TSlots: GetItem<SlotKey> + ThreadSafe,
		TSlots::TItem: InspectAble<ItemDescription>;
}

pub trait SwapValuesByKey {
	fn swap(&mut self, a: SwapKey, b: SwapKey);
}

pub trait GetItem<TKey> {
	type TItem;

	fn get_item(&self, key: TKey) -> Option<&Self::TItem>;
}
