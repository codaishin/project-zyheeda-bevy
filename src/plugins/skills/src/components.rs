pub mod combo_node;
pub mod combos;
pub mod combos_time_out;
pub mod inventory;
pub mod queue;
pub mod slots;

pub(crate) mod skill_executer;

use self::slots::Slots;
use crate::{
	items::{slot_key::SlotKey, Item},
	skills::Skill,
};
use bevy::{
	ecs::{component::Component, entity::Entity},
	utils::default,
};
use common::{components::Collection, traits::load_asset::Path};
use std::collections::HashMap;

#[derive(PartialEq, Debug, Clone)]
pub struct Mounts<T> {
	pub hand: T,
	pub forearm: T,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Slot<TSkill = Skill> {
	pub mounts: Mounts<Entity>,
	pub item: Option<Item<TSkill>>,
}

pub(crate) type BoneName = str;
pub(crate) type SlotContent = (Mounts<&'static BoneName>, Option<Item<Path>>);
pub(crate) type SlotDefinition = (SlotKey, SlotContent);

#[derive(Component, Clone, PartialEq, Debug)]
pub(crate) struct SlotsDefinition {
	pub(crate) definitions: HashMap<SlotKey, (Mounts<&'static BoneName>, Option<Item<Path>>)>,
	pub(crate) slot_buffer: Slots<Path>,
}

impl SlotsDefinition {
	pub(crate) fn new<const N: usize>(definitions: [SlotDefinition; N]) -> Self {
		Self {
			definitions: definitions.into(),
			slot_buffer: default(),
		}
	}
}

#[derive(Component, Debug, PartialEq)]
pub(crate) struct SkillSpawn<T>(pub T);

#[derive(Debug, PartialEq)]
pub(crate) struct LoadModel(pub SlotKey);

pub(crate) type LoadModelsCommand = Collection<LoadModel>;
