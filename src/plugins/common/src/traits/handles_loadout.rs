pub mod combos_component;
pub mod inventory_component;
pub mod loadout;
pub mod slot_component;

use crate::{
	tools::{
		action_key::slot::{PlayerSlot, SlotKey},
		inventory_key::InventoryKey,
		skill_execution::SkillExecution,
	},
	traits::{
		accessors::get::{EntityContext, EntityContextMut, GetProperty},
		handles_loadout::{
			combos_component::{Combo, CombosComponent, GetCombosOrdered, NextConfiguredKeys},
			inventory_component::InventoryComponent,
			loadout::{
				ItemToken,
				LoadoutSkill,
				LoadoutSkillItem,
				SkillIcon,
				SkillToken,
				SwapExternal,
			},
			slot_component::SlotComponent,
		},
		thread_safe::ThreadSafe,
	},
};
use bevy::{
	ecs::{component::Mutable, system::SystemParam},
	prelude::*,
};
use std::{fmt::Debug, ops::DerefMut};

pub trait HandlesLoadout {
	type TItem: LoadoutSkillItem;
	type TSkill: LoadoutSkill;
	type TSkills: IntoIterator<Item = Self::TSkill>;

	type TInventory: InventoryComponent<Self::TItem> + SwapExternal<Self::TSlots>;
	type TSlots: SlotComponent<Self::TItem, Self::TSkills> + SwapExternal<Self::TInventory>;
	type TCombos: CombosComponent<Self::TSkill>;
}

pub trait HandlesLoadout2 {
	type TSkillID: Debug + PartialEq + Copy + ThreadSafe;

	type TLoadoutRead<'w, 's>: SystemParam
		+ for<'c> EntityContext<Items, TContext<'c>: ReadItems>
		+ for<'c> EntityContext<Skills, TContext<'c>: ReadSkills>
		+ for<'c> EntityContext<Combos, TContext<'c>: ReadCombos<Self::TSkillID>>
		+ for<'c> EntityContext<AvailableSkills, TContext<'c>: ReadAvailableSkills<Self::TSkillID>>;

	type TLoadoutMut<'w, 's>: SystemParam
		+ for<'c> EntityContextMut<Items, TContext<'c>: SwapItems>
		+ for<'c> EntityContextMut<Combos, TContext<'c>: UpdateCombos2<Self::TSkillID>>;
}

pub type LoadoutReadParam<'w, 's, T> = <T as HandlesLoadout2>::TLoadoutRead<'w, 's>;
pub type LoadoutMutParam<'w, 's, T> = <T as HandlesLoadout2>::TLoadoutMut<'w, 's>;

pub struct Items;

pub trait ReadItems {
	type TItem<'a>: GetProperty<ItemToken>
	where
		Self: 'a;

	fn get_item<TKey>(&self, key: TKey) -> Option<Self::TItem<'_>>
	where
		TKey: Into<LoadoutKey>;
}

pub trait SwapItems {
	fn swap_items<TA, TB>(&mut self, a: TA, b: TB)
	where
		TA: Into<LoadoutKey>,
		TB: Into<LoadoutKey>;
}

pub struct Skills;

pub trait ReadSkills {
	type TSkill<'a>: GetProperty<SkillToken> + GetProperty<SkillIcon> + GetProperty<SkillExecution>
	where
		Self: 'a;

	fn get_skill<TKey>(&self, key: TKey) -> Option<Self::TSkill<'_>>
	where
		TKey: Into<LoadoutKey>;
}

pub struct AvailableSkills;

pub trait GetSkillId<TSkillId> {
	fn get_skill_id(&self) -> TSkillId;
}

pub trait ReadAvailableSkills<TSkillID> {
	type TSkill<'a>: GetProperty<SkillToken> + GetProperty<SkillIcon> + GetSkillId<TSkillID>
	where
		Self: 'a;

	fn get_available_skills(&self, key: SlotKey) -> impl Iterator<Item = Self::TSkill<'_>>;
}

pub struct Combos;

pub trait ReadCombos<TId>:
	GetCombosOrdered<
		TKey = SlotKey,
		TItem: GetSkillId<TId> + GetProperty<SkillToken> + GetProperty<SkillIcon>,
	> + NextConfiguredKeys<SlotKey>
{
}

impl<T, TId> ReadCombos<TId> for T where
	T: GetCombosOrdered<
			TKey = SlotKey,
			TItem: GetSkillId<TId> + GetProperty<SkillToken> + GetProperty<SkillIcon>,
		> + NextConfiguredKeys<SlotKey>
{
}

pub trait UpdateCombos2<TSkillID> {
	fn update_combos(&mut self, combos: Combo<SlotKey, Option<TSkillID>>);
}

impl<T, TSkillID> UpdateCombos2<TSkillID> for Mut<'_, T>
where
	T: Component<Mutability = Mutable> + UpdateCombos2<TSkillID>,
{
	fn update_combos(&mut self, combos: Combo<SlotKey, Option<TSkillID>>) {
		self.deref_mut().update_combos(combos)
	}
}

impl<T, TSkillID> UpdateCombos2<TSkillID> for ResMut<'_, T>
where
	T: Resource + UpdateCombos2<TSkillID>,
{
	fn update_combos(&mut self, combos: Combo<SlotKey, Option<TSkillID>>) {
		self.deref_mut().update_combos(combos)
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum LoadoutKey {
	Inventory(InventoryKey),
	Slot(SlotKey),
}

impl From<InventoryKey> for LoadoutKey {
	fn from(key: InventoryKey) -> Self {
		Self::Inventory(key)
	}
}

impl From<SlotKey> for LoadoutKey {
	fn from(key: SlotKey) -> Self {
		Self::Slot(key)
	}
}

impl From<PlayerSlot> for LoadoutKey {
	fn from(key: PlayerSlot) -> Self {
		Self::Slot(SlotKey::from(key))
	}
}
