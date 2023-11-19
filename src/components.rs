pub mod lazy;
pub mod marker;

use crate::{
	behaviors::{Behavior, MovementMode},
	types::BoneName,
};
use bevy::{
	prelude::{Component, *},
	utils::HashMap,
};
use std::{collections::VecDeque, fmt::Debug, marker::PhantomData, time::Duration};

use self::{lazy::Lazy, marker::MarkerCommands};

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

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Cast {
	pub pre: Duration,
	pub after: Duration,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Agent(pub Entity);

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Spawner(pub GlobalTransform);

pub type SpawnBehaviorFn = fn(&mut Commands, Agent, Spawner, Ray);

#[derive(Component, PartialEq, Debug)]
pub struct Skill {
	pub ray: Ray,
	pub cast: Cast,
	pub marker_commands: MarkerCommands,
	pub behavior: Lazy,
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
	pub behaviors: HashMap<SlotKey, Behavior>,
}

#[derive(PartialEq, Debug)]
pub struct Slot {
	pub entity: Entity,
	pub behavior: Option<Behavior>,
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
	pub behavior: Option<Behavior>,
}

#[derive(Component, Debug, PartialEq)]
pub struct Equip(pub Vec<Item>);

impl Equip {
	pub fn new<const N: usize>(items: [Item; N]) -> Self {
		Self(items.into())
	}
}

#[derive(Component)]
pub struct Queue(pub VecDeque<(Behavior, Ray)>);

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
