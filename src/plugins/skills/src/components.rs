pub mod queue;

use crate::skill::{Skill, SkillComboTree, StartBehaviorFn, StopBehaviorFn};
use bevy::ecs::{component::Component, entity::Entity};
use common::{
	components::{Collection, Side},
	traits::look_up::LookUp,
};
use std::{
	collections::{HashMap, HashSet},
	fmt::{Display, Formatter, Result},
};

#[derive(Component, Clone)]
pub(crate) struct ComboTreeTemplate<TNext>(pub HashMap<SlotKey, SkillComboTree<TNext>>);

#[derive(Component, PartialEq, Debug)]
pub(crate) struct ComboTreeRunning<TNext>(pub HashMap<SlotKey, SkillComboTree<TNext>>);

#[derive(PartialEq, Debug, Clone)]
pub struct Slot {
	pub entity: Entity,
	pub item: Option<Item>,
}

pub(crate) type BoneName = str;

#[derive(Component, Clone, PartialEq, Debug)]
pub struct SlotBones(pub HashMap<SlotKey, &'static BoneName>);

#[derive(Component)]
pub struct Slots(pub HashMap<SlotKey, Slot>);

impl Slots {
	pub fn new() -> Self {
		Self(HashMap::new())
	}
}

impl Default for Slots {
	fn default() -> Self {
		Self::new()
	}
}

impl LookUp<SlotKey, Skill> for Slots {
	fn get(&self, key: &SlotKey) -> Option<&Skill> {
		let slot = self.0.get(key)?;
		let item = slot.item.as_ref()?;
		item.skill.as_ref()
	}
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Item {
	pub name: &'static str,
	pub model: Option<&'static str>,
	pub skill: Option<Skill>,
	pub item_type: HashSet<ItemType>,
}

impl Display for Item {
	fn fmt(&self, f: &mut Formatter) -> Result {
		match &self.skill {
			Some(skill) => write!(f, "Item({}, {})", self.name, skill),
			None => write!(f, "Item({}, <no skill>)", self.name),
		}
	}
}

pub type Inventory = Collection<Option<Item>>;
pub type Equipment = Collection<(SlotKey, Option<Item>)>;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct InventoryKey(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Handed<TSide> {
	Single(TSide),
	Dual(TSide),
}

impl Handed<SideUnset> {
	pub fn on(self, side: Side) -> Handed<Side> {
		match self {
			Handed::Single(_) => Handed::Single(side),
			Handed::Dual(_) => Handed::Dual(side),
		}
	}
}

#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug)]
pub struct SideUnset;

#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug, Default)]
pub enum SlotKey {
	#[default]
	SkillSpawn,
	Hand(Side),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum ItemType {
	Pistol,
	Sword,
	Legs,
}

#[derive(Component, Debug, PartialEq)]
pub(crate) enum SlotVisibility {
	Inherited(Vec<SlotKey>),
	Hidden(Vec<SlotKey>),
}

#[derive(Component, Debug, PartialEq)]
pub(crate) enum SkillExecution {
	Start(StartBehaviorFn),
	Stop(StopBehaviorFn),
}
