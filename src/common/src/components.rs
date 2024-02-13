use crate::{
	behaviors::MovementMode,
	skill::{PlayerSkills, Queued, Skill},
	tools::UnitsPerSecond,
};
use bevy::ecs::{component::Component, entity::Entity};
use std::{
	collections::{HashMap, HashSet, VecDeque},
	fmt::{Display, Formatter, Result},
	time::Duration,
};

#[derive(Debug, PartialEq)]
pub struct Swap<T1, T2>(pub T1, pub T2);

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
pub enum Side {
	Main,
	Off,
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

#[derive(PartialEq, Debug, Clone)]
pub struct Slot {
	pub entity: Entity,
	pub item: Option<Item>,
	pub combo_skill: Option<Skill>,
}

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

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum ItemType {
	Pistol,
	Sword,
	Legs,
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

#[derive(Component, Debug, PartialEq)]
pub struct Collection<TElement>(pub Vec<TElement>);

impl<TElement> Collection<TElement> {
	pub fn new<const N: usize>(elements: [TElement; N]) -> Self {
		Self(elements.into())
	}
}

pub type Inventory = Collection<Option<Item>>;
pub type Equipment = Collection<(SlotKey, Option<Item>)>;

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

#[derive(Component)]
pub struct Queue<TAnimationKey = PlayerSkills<SideUnset>>(
	pub VecDeque<Skill<TAnimationKey, Queued>>,
);

impl Default for Queue {
	fn default() -> Self {
		Self(Default::default())
	}
}

#[derive(Component, Default)]
pub struct Player {
	pub walk_speed: UnitsPerSecond,
	pub run_speed: UnitsPerSecond,
	pub movement_mode: MovementMode,
}

#[derive(Component)]
pub struct VoidSphere;

#[derive(Component, Debug, PartialEq)]
pub struct Health {
	pub current: i16,
	pub max: i16,
}

impl Health {
	pub fn new(value: i16) -> Self {
		Self {
			current: value,
			max: value,
		}
	}
}

#[derive(Component, PartialEq, Debug)]
pub struct ColliderRoot(pub Entity);

#[derive(Component)]
pub struct DealsDamage(pub i16);
