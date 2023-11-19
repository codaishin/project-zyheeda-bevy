pub mod marker;

use crate::{
	behaviors::{meta::BehaviorMeta, MovementMode},
	types::BoneName,
};
use bevy::{
	prelude::{Component, *},
	utils::HashMap,
};
use std::{collections::VecDeque, fmt::Debug, marker::PhantomData, time::Duration};

use self::marker::Markers;

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

#[derive(PartialEq, Debug, Clone, Copy, Default)]
pub struct Cast {
	pub pre: Duration,
	pub after: Duration,
}

#[derive(Component, PartialEq, Debug, Clone, Default)]
pub struct Skill<TData = ()> {
	pub data: TData,
	pub cast: Cast,
	pub markers: Markers,
	pub behavior: BehaviorMeta,
}

impl Skill {
	pub fn with<T: Clone + Copy>(self, data: T) -> Skill<T> {
		Skill {
			data,
			cast: self.cast,
			markers: self.markers,
			behavior: self.behavior,
		}
	}
}

#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug)]
pub enum Side {
	Right,
	Left,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug)]
pub enum SlotKey {
	SkillSpawn,
	Hand(Side),
	Legs,
}

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

#[derive(PartialEq, Debug)]
pub struct Slot {
	pub entity: Entity,
	pub skill: Option<Skill>,
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

#[derive(Debug, PartialEq, Clone)]
pub struct Item {
	pub slot: SlotKey,
	pub model: Option<&'static str>,
	pub skill: Option<Skill>,
}

#[derive(Component, Debug, PartialEq)]
pub struct Equip(pub Vec<Item>);

impl Equip {
	pub fn new<const N: usize>(items: [Item; N]) -> Self {
		Self(items.into())
	}
}

#[derive(Component)]
pub struct Queue(pub VecDeque<Skill<Ray>>);

#[derive(Component)]
pub struct TimeTracker<TBehavior> {
	pub duration: Duration,
	phantom_data: PhantomData<TBehavior>,
}

impl<T> TimeTracker<T> {
	pub fn new() -> Self {
		Self {
			duration: Duration::ZERO,
			phantom_data: PhantomData,
		}
	}
}

impl<T> Default for TimeTracker<T> {
	fn default() -> Self {
		Self::new()
	}
}

#[derive(Component)]
pub struct Projectile {
	pub target_ray: Ray,
	pub range: f32,
}
