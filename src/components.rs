use crate::{behaviors::MovementMode, skill::Skill, types::BoneName};
use bevy::prelude::{Component, *};
use core::fmt::Display;
use std::{
	collections::{HashMap, VecDeque},
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
pub struct WaitNext;

#[derive(Component, Clone, Copy, PartialEq, Debug)]
pub struct SimpleMovement {
	pub target: Vec3,
}

impl SimpleMovement {
	pub fn new(target: Vec3) -> Self {
		Self { target }
	}
}

#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub struct Queued {
	pub ray: Ray,
	pub slot: SlotKey,
}

#[derive(PartialEq, Debug, Clone, Copy, Default)]
pub struct Active {
	pub ray: Ray,
	pub slot: SlotKey,
}

impl Skill {
	pub fn with<T: Clone + Copy>(self, data: T) -> Skill<T> {
		Skill {
			data,
			name: self.name,
			cast: self.cast,
			soft_override: self.soft_override,
			marker: self.marker,
			behavior: self.behavior,
		}
	}
}

impl<TSrc> Skill<TSrc> {
	pub fn map_data<TDst>(self, map: fn(TSrc) -> TDst) -> Skill<TDst> {
		Skill {
			name: self.name,
			data: map(self.data),
			cast: self.cast,
			marker: self.marker,
			soft_override: self.soft_override,
			behavior: self.behavior,
		}
	}
}

impl Skill<Queued> {
	pub fn to_active(self) -> Skill<Active> {
		self.map_data(|Queued { ray, slot }| Active { ray, slot })
	}
}

#[derive(Component, Debug, PartialEq)]
pub struct Marker<T> {
	phantom_data: PhantomData<T>,
}

impl<T> Marker<T> {
	pub fn new() -> Self {
		Self {
			phantom_data: PhantomData,
		}
	}
}

impl<T> Default for Marker<T> {
	fn default() -> Self {
		Self::new()
	}
}

#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug)]
pub enum Side {
	Right,
	Left,
}

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
	pub alternative_skill: Option<Skill>,
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

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Item {
	pub name: &'static str,
	pub model: Option<&'static str>,
	pub skill: Option<Skill>,
}

impl Display for Item {
	fn fmt(&self, f: &mut Formatter) -> Result {
		match self.skill {
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
pub struct Queue(pub VecDeque<Skill<Queued>>);

#[derive(Component)]
pub struct Projectile {
	pub target_ray: Ray,
	pub range: f32,
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

impl<T: Copy> Track<T> {
	pub fn new(value: T) -> Self {
		Self {
			value,
			duration: Duration::ZERO,
		}
	}
}
