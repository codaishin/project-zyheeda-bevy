pub mod available_skills;
pub mod combos;
pub mod items;
pub mod skills;

use crate::{
	tools::{
		action_key::slot::{PlayerSlot, SlotKey},
		inventory_key::InventoryKey,
	},
	traits::{
		accessors::get::{EntityContext, EntityContextMut},
		handles_loadout::{
			available_skills::{AvailableSkills, ReadAvailableSkills},
			combos::{Combos, ReadCombos, UpdateCombos},
			items::{Items, ReadItems, SwapItems},
			skills::{ReadSkills, Skills},
		},
		thread_safe::ThreadSafe,
	},
};
use bevy::ecs::system::SystemParam;
use std::fmt::Debug;

pub trait HandlesLoadout {
	type TSkillID: Debug + PartialEq + Copy + ThreadSafe;

	type TLoadoutRead<'w, 's>: SystemParam
		+ for<'c> EntityContext<Items, TContext<'c>: ReadItems>
		+ for<'c> EntityContext<Skills, TContext<'c>: ReadSkills>
		+ for<'c> EntityContext<Combos, TContext<'c>: ReadCombos<Self::TSkillID>>
		+ for<'c> EntityContext<AvailableSkills, TContext<'c>: ReadAvailableSkills<Self::TSkillID>>;

	type TLoadoutMut<'w, 's>: SystemParam
		+ for<'c> EntityContextMut<Items, TContext<'c>: SwapItems>
		+ for<'c> EntityContextMut<Combos, TContext<'c>: UpdateCombos<Self::TSkillID>>;
}

pub type LoadoutReadParam<'w, 's, T> = <T as HandlesLoadout>::TLoadoutRead<'w, 's>;
pub type LoadoutMutParam<'w, 's, T> = <T as HandlesLoadout>::TLoadoutMut<'w, 's>;

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
