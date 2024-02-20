use crate::skill::{PlayerSkills, Queued, Skill, SkillComboTree};
use bevy::ecs::{component::Component, entity::Entity};
use common::components::{Collection, Side};
use std::{
	collections::{HashMap, HashSet, VecDeque},
	fmt::{Display, Formatter, Result},
	time::Duration,
};

#[derive(Component, Clone)]
pub(crate) struct ComboTreeTemplate<TNext>(pub HashMap<SlotKey, SkillComboTree<TNext>>);

#[derive(Component, PartialEq, Debug)]
pub(crate) struct ComboTreeRunning<TNext>(pub HashMap<SlotKey, SkillComboTree<TNext>>);

#[derive(PartialEq, Debug, Clone)]
pub struct Slot {
	pub entity: Entity,
	pub item: Option<Item>,
	pub combo_skill: Option<Skill>,
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

#[derive(Component)]
pub struct Queue<TAnimationKey = PlayerSkills<SideUnset>>(
	pub VecDeque<Skill<TAnimationKey, Queued>>,
);

impl Default for Queue {
	fn default() -> Self {
		Self(Default::default())
	}
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct InventoryKey(pub usize);

#[derive(Component, Debug, PartialEq)]
pub struct Track<T> {
	pub value: T,
	pub elapsed: Duration,
}

impl<T: Clone> Track<T> {
	pub fn new(value: T) -> Self {
		Self {
			value: value.clone(),
			elapsed: Duration::ZERO,
		}
	}
}

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
	Legs,
	SkillSpawn,
	Hand(Side),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum ItemType {
	Pistol,
	Sword,
	Legs,
}

#[derive(Component, PartialEq, Debug)]
pub(crate) enum Schedule {
	Enqueue((SlotKey, Skill)),
	Override((SlotKey, Skill)),
	StopAimAfter(Duration),
	UpdateTarget,
}
