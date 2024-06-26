pub mod combos;
pub mod combos_time_out;
pub mod inventory;
pub mod queue;
pub mod slots;

pub(crate) mod skill_executer;

use self::slots::Slots;
use crate::items::{slot_key::SlotKey, Item};
use bevy::ecs::{component::Component, entity::Entity};
use common::components::Collection;
use std::collections::HashMap;

#[derive(PartialEq, Debug, Clone)]
pub struct Mounts<T> {
	pub hand: T,
	pub forearm: T,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Slot {
	pub mounts: Mounts<Entity>,
	pub item: Option<Item>,
}

pub(crate) type BoneName = str;

#[derive(Component, Clone, PartialEq, Debug)]
pub struct SlotBones(pub HashMap<SlotKey, Mounts<&'static BoneName>>);

#[derive(Component, Debug, PartialEq)]
pub(crate) struct SkillSpawn<T>(pub T);

pub type Equipment = Collection<(SlotKey, Option<Item>)>;
