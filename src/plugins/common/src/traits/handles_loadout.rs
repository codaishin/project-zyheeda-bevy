pub mod available_skills;
pub mod combos;
pub mod insert_default_loadout;
pub mod items;
pub mod register_loadout_bones;
pub mod skills;

use crate::{
	tools::{
		action_key::slot::{PlayerSlot, SlotKey},
		inventory_key::InventoryKey,
	},
	traits::{
		accessors::get::{GetContext, GetContextMut},
		handles_loadout::{
			available_skills::{AvailableSkills, ReadAvailableSkills},
			combos::{Combos, ReadCombos, UpdateCombos},
			insert_default_loadout::{InsertDefaultLoadout, NotLoadedOut},
			items::{Items, ReadItems, SwapItems},
			register_loadout_bones::{NoBonesRegistered, RegisterLoadoutBones},
			skills::{ReadSkills, Skills},
		},
		thread_safe::ThreadSafe,
	},
};
use bevy::ecs::system::SystemParam;
use std::fmt::Debug;

pub trait HandlesLoadout {
	type TSkillID: Debug + PartialEq + Copy + ThreadSafe;

	type TLoadoutPrep<'w, 's>: SystemParam
		+ for<'c> GetContextMut<NotLoadedOut, TContext<'c>: InsertDefaultLoadout>
		+ for<'c> GetContextMut<NoBonesRegistered, TContext<'c>: RegisterLoadoutBones>;

	type TLoadoutRead<'w, 's>: SystemParam
		+ for<'c> GetContext<Items, TContext<'c>: ReadItems>
		+ for<'c> GetContext<Skills, TContext<'c>: ReadSkills>
		+ for<'c> GetContext<Combos, TContext<'c>: ReadCombos<Self::TSkillID>>
		+ for<'c> GetContext<AvailableSkills, TContext<'c>: ReadAvailableSkills<Self::TSkillID>>;

	type TLoadoutMut<'w, 's>: SystemParam
		+ for<'c> GetContextMut<Items, TContext<'c>: SwapItems>
		+ for<'c> GetContextMut<Combos, TContext<'c>: UpdateCombos<Self::TSkillID>>;

	type TLoadoutActivity<'w, 's>: SystemParam
		+ for<'c> GetContext<Skills, TContext<'c>: ActiveSkills>;
}

pub type LoadoutPrepParam<'w, 's, T> = <T as HandlesLoadout>::TLoadoutPrep<'w, 's>;
pub type LoadoutReadParam<'w, 's, T> = <T as HandlesLoadout>::TLoadoutRead<'w, 's>;
pub type LoadoutMutParam<'w, 's, T> = <T as HandlesLoadout>::TLoadoutMut<'w, 's>;
pub type LoadoutActivityParam<'w, 's, T> = <T as HandlesLoadout>::TLoadoutActivity<'w, 's>;

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

pub trait ActiveSkills {
	type TIter<'a>: Iterator<Item = ActiveSkill>
	where
		Self: 'a;

	fn active_skills(&self) -> Self::TIter<'_>;
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ActiveSkill {
	pub key: SlotKey,
	pub animate: bool,
}
