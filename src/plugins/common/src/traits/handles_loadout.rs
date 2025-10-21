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
use std::{
	fmt::Debug,
	ops::{Deref, DerefMut},
};

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

impl<T> ReadItems for T
where
	T: Deref<Target: ReadItems>,
{
	type TItem<'a>
		= <<T as Deref>::Target as ReadItems>::TItem<'a>
	where
		Self: 'a;

	fn get_item<TKey>(&self, key: TKey) -> Option<Self::TItem<'_>>
	where
		TKey: Into<LoadoutKey>,
	{
		self.deref().get_item(key)
	}
}

pub trait SwapItems {
	fn swap_items<TA, TB>(&mut self, a: TA, b: TB)
	where
		TA: Into<LoadoutKey>,
		TB: Into<LoadoutKey>;
}

impl<T> SwapItems for T
where
	T: DerefMut<Target: SwapItems>,
{
	fn swap_items<TA, TB>(&mut self, a: TA, b: TB)
	where
		TA: Into<LoadoutKey>,
		TB: Into<LoadoutKey>,
	{
		self.deref_mut().swap_items(a, b);
	}
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

impl<T> ReadSkills for T
where
	T: Deref<Target: ReadSkills>,
{
	type TSkill<'a>
		= <<T as Deref>::Target as ReadSkills>::TSkill<'a>
	where
		Self: 'a;

	fn get_skill<TKey>(&self, key: TKey) -> Option<Self::TSkill<'_>>
	where
		TKey: Into<LoadoutKey>,
	{
		self.deref().get_skill(key)
	}
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

impl<T, TSkillID> ReadAvailableSkills<TSkillID> for T
where
	T: Deref<Target: ReadAvailableSkills<TSkillID>>,
{
	type TSkill<'a>
		= <<T as Deref>::Target as ReadAvailableSkills<TSkillID>>::TSkill<'a>
	where
		Self: 'a;

	fn get_available_skills(&self, key: SlotKey) -> impl Iterator<Item = Self::TSkill<'_>> {
		self.deref().get_available_skills(key)
	}
}

pub struct Combos;

pub trait ReadCombos<TId>:
	GetCombosOrdered<TSkill: GetSkillId<TId> + GetProperty<SkillToken> + GetProperty<SkillIcon>>
	+ NextConfiguredKeys<SlotKey>
{
}

impl<T, TId> ReadCombos<TId> for T where
	T: GetCombosOrdered<TSkill: GetSkillId<TId> + GetProperty<SkillToken> + GetProperty<SkillIcon>>
		+ NextConfiguredKeys<SlotKey>
{
}

pub trait UpdateCombos2<TSkillID> {
	fn update_combos(&mut self, combos: Combo<SlotKey, Option<TSkillID>>);
}

impl<T, TSkillID> UpdateCombos2<TSkillID> for T
where
	T: DerefMut<Target: Component<Mutability = Mutable> + UpdateCombos2<TSkillID>>,
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
