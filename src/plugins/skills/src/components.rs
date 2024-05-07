pub mod combos;
pub mod inventory;
pub mod queue;
pub mod slots;

use self::slots::Slots;
use crate::{
	items::{Item, SlotKey},
	skills::{StartBehaviorFn, StopBehaviorFn},
};
use bevy::ecs::{component::Component, entity::Entity};
use common::components::Collection;
use std::collections::HashMap;

#[derive(PartialEq, Debug, Clone)]
pub struct Slot {
	pub entity: Entity,
	pub item: Option<Item>,
}

pub(crate) type BoneName = str;

#[derive(Component, Clone, PartialEq, Debug)]
pub struct SlotBones(pub HashMap<SlotKey, &'static BoneName>);

pub type Equipment = Collection<(SlotKey, Option<Item>)>;

#[derive(Component, Debug, PartialEq)]
pub(crate) enum SkillExecution {
	Start(StartBehaviorFn),
	Stop(StopBehaviorFn),
}
