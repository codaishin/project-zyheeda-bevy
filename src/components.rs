use crate::{
	behaviors::MovementMode,
	skill::{Queued, Skill, SkillComboTree},
	types::BoneName,
};
use bevy::prelude::{Component, Entity, Vec3};
use core::fmt::Display;
use std::{
	collections::{HashMap, HashSet, VecDeque},
	fmt::{Debug, Formatter, Result},
	marker::PhantomData,
	time::Duration,
};

#[derive(Component)]
pub struct CamOrbit {
	pub center: Vec3,
	pub distance: f32,
	pub sensitivity: f32,
}

/// Represents units per second.
/// Is clamped at minimum 0.
#[derive(PartialEq, PartialOrd, Clone, Copy, Debug, Default)]
pub struct UnitsPerSecond(f32);

impl UnitsPerSecond {
	pub fn new(value: f32) -> Self {
		if value < 0. {
			Self(0.)
		} else {
			Self(value)
		}
	}

	pub fn to_f32(self) -> f32 {
		self.0
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn set_value() {
		let speed = UnitsPerSecond::new(42.);

		assert_eq!(42., speed.to_f32());
	}

	#[test]
	fn min_zero() {
		let speed = UnitsPerSecond::new(-42.);

		assert_eq!(0., speed.to_f32());
	}
}

#[derive(Component, Default)]
pub struct Player {
	pub walk_speed: UnitsPerSecond,
	pub run_speed: UnitsPerSecond,
	pub movement_mode: MovementMode,
}

#[derive(Component, Default)]
pub struct Animator {
	pub animation_player_id: Option<Entity>,
}

#[derive(Component)]
pub struct DequeueNext;

#[derive(Component, Clone, Copy, PartialEq, Debug)]
pub struct SimpleMovement {
	pub target: Vec3,
}

impl SimpleMovement {
	pub fn new(target: Vec3) -> Self {
		Self { target }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlayerSkills<TSide> {
	#[default]
	Idle,
	Shoot(Handed<TSide>),
	SwordStrike(TSide),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayerMovement {
	Walk,
	Run,
}

#[derive(Component, PartialEq, Debug, Clone, Copy, Default)]
pub enum Animate<T: Copy + Clone> {
	#[default]
	None,
	Replay(T),
	Repeat(T),
}

#[derive(Component)]
pub struct Mark<T>(pub T);

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

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct InventoryKey(pub usize);

#[derive(Component, Clone, PartialEq, Debug)]
pub struct SlotBones(pub HashMap<SlotKey, &'static BoneName>);

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum ScheduleMode {
	Enqueue,
	Override,
}

#[derive(Component, PartialEq, Debug)]
pub struct Schedule {
	pub mode: ScheduleMode,
	pub skills: HashMap<SlotKey, Skill>,
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

#[derive(Component)]
pub struct Queue<TAnimationKey = PlayerSkills<SideUnset>>(
	pub VecDeque<Skill<TAnimationKey, Queued>>,
);

impl Default for Queue {
	fn default() -> Self {
		Self(Default::default())
	}
}

pub struct Plasma;

#[derive(Component, Default)]
pub struct Projectile<T> {
	pub direction: Vec3,
	pub range: f32,
	phantom_data: PhantomData<T>,
}

impl<T> Projectile<T> {
	pub fn new(direction: Vec3, range: f32) -> Self {
		Self {
			direction,
			range,
			phantom_data: PhantomData,
		}
	}
}

#[derive(Debug, PartialEq)]
pub struct Swap<T1, T2>(pub T1, pub T2);

#[derive(Component, Debug, Clone, Copy)]
pub struct KeyedPanel<TKey>(pub TKey);

#[derive(Component, Debug, PartialEq, Clone, Copy)]
pub struct Dad<T>(pub T);

#[derive(Component, Debug, PartialEq)]
pub struct Track<T> {
	pub value: T,
	pub duration: Duration,
}

impl<T: Clone> Track<T> {
	pub fn new(value: T) -> Self {
		Self {
			value: value.clone(),
			duration: Duration::ZERO,
		}
	}
}

#[derive(Component, Clone)]
pub struct ComboTreeTemplate<TNext>(pub HashMap<SlotKey, SkillComboTree<TNext>>);

#[derive(Component, PartialEq, Debug)]
pub struct ComboTreeRunning<TNext>(pub HashMap<SlotKey, SkillComboTree<TNext>>);

#[derive(Component)]
pub struct Dummy;

#[derive(Component, PartialEq, Debug)]
pub struct ColliderRoot(pub Entity);
