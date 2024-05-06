pub mod combos;
pub mod inventory;
pub mod queue;
pub mod slots;

use self::slots::Slots;
use crate::{
	items::{Item, SlotKey},
	skills::{Skill, StartBehaviorFn, StopBehaviorFn},
};
use bevy::ecs::{component::Component, entity::Entity};
use common::{components::Collection, traits::look_up::LookUp};
use std::collections::HashMap;

#[derive(PartialEq, Debug, Clone)]
pub struct Slot {
	pub entity: Entity,
	pub item: Option<Item>,
}

pub(crate) type BoneName = str;

#[derive(Component, Clone, PartialEq, Debug)]
pub struct SlotBones(pub HashMap<SlotKey, &'static BoneName>);

impl LookUp<SlotKey, Skill> for Slots {
	fn get(&self, key: &SlotKey) -> Option<&Skill> {
		let slot = self.0.get(key)?;
		let item = slot.item.as_ref()?;
		item.skill.as_ref()
	}
}

pub type Equipment = Collection<(SlotKey, Option<Item>)>;

#[derive(Component, Debug, PartialEq)]
pub(crate) enum SkillExecution {
	Start(StartBehaviorFn),
	Stop(StopBehaviorFn),
}
