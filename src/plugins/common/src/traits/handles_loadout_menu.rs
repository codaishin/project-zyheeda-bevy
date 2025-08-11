use super::{inspect_able::InspectAble, thread_safe::ThreadSafe};
use crate::tools::{
	action_key::slot::PlayerSlot,
	change::Change,
	inventory_key::InventoryKey,
	item_description::ItemToken,
	skill_description::SkillToken,
	skill_execution::SkillExecution,
	skill_icon::SkillIcon,
	swap_key::SwapKey,
};
use bevy::{ecs::component::Mutable, prelude::*};

pub trait HandlesLoadoutMenu {
	fn loadout_with_swapper<TSwap>() -> impl ConfigureInventory<TSwap>
	where
		TSwap: Component<Mutability = Mutable> + SwapValuesByKey;

	fn configure_quickbar_menu<TQuickbar, TSystemMarker>(
		app: &mut App,
		get_changed_quickbar: impl IntoSystem<(), Change<TQuickbar>, TSystemMarker>,
	) where
		TQuickbar: GetItem<PlayerSlot> + ThreadSafe,
		TQuickbar::TItem:
			InspectAble<SkillToken> + InspectAble<SkillIcon> + InspectAble<SkillExecution>;
}

pub trait ConfigureInventory<TSwap> {
	fn configure<TInventory, TSlots, TSystemMarker1, TSystemMarker2>(
		&self,
		app: &mut App,
		get_changed_inventory: impl IntoSystem<(), Change<TInventory>, TSystemMarker1>,
		get_changed_slots: impl IntoSystem<(), Change<TSlots>, TSystemMarker2>,
	) where
		TInventory: GetItem<InventoryKey> + ThreadSafe,
		TInventory::TItem: InspectAble<ItemToken>,
		TSlots: GetItem<PlayerSlot> + ThreadSafe,
		TSlots::TItem: InspectAble<ItemToken>;
}

pub trait SwapValuesByKey {
	fn swap(&mut self, a: SwapKey, b: SwapKey);
}

pub trait GetItem<TKey> {
	type TItem;

	fn get_item(&self, key: TKey) -> Option<&Self::TItem>;
}
