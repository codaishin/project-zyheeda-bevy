use crate::{behavior::MovementMode, types::BoneName};
use bevy::{prelude::*, utils::HashMap};
use std::{borrow::Cow, collections::VecDeque, fmt::Debug};

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
pub struct Queue<T>(pub VecDeque<T>);

impl<T> Queue<T> {
	pub fn new() -> Self {
		Self([].into())
	}
}

impl<T> Default for Queue<T> {
	fn default() -> Self {
		Self::new()
	}
}

#[derive(Component)]
pub struct Idle;

#[derive(Component, PartialEq, Debug)]
pub struct Walk;

#[derive(Component, PartialEq, Debug)]
pub struct Run;

#[derive(Component, Clone, Copy, PartialEq, Debug)]
pub struct SimpleMovement {
	pub target: Vec3,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug)]
pub enum Side {
	Right,
	Left,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug)]
pub enum SlotKey {
	Hand(Side),
	Legs,
}

#[derive(Component, Clone, PartialEq, Debug)]
pub struct SlotInfos(pub HashMap<SlotKey, Cow<'static, BoneName>>);

impl SlotInfos {
	pub fn new<const C: usize>(pairs: [(SlotKey, &'static BoneName); C]) -> Self {
		Self(pairs.map(|(k, v)| (k, Cow::from(v))).into())
	}
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Schedule {
	Enqueue,
	Override,
}

type GetBehaviorFn<TBehavior> = Option<fn(ray: Ray) -> Option<TBehavior>>;

#[derive(PartialEq, Debug)]
pub struct Slot<TBehavior> {
	pub entity: Entity,
	pub schedule: Option<Schedule>,
	pub get_behavior: GetBehaviorFn<TBehavior>,
}

impl<TBehavior> Slot<TBehavior> {
	pub fn new(entity: Entity, behavior: GetBehaviorFn<TBehavior>) -> Self {
		Self {
			entity,
			get_behavior: behavior,
			schedule: None,
		}
	}
}

#[derive(Component)]
pub struct Slots<TBehavior>(pub HashMap<SlotKey, Slot<TBehavior>>);

impl<TBehavior> Slots<TBehavior> {
	pub fn new() -> Self {
		Self(HashMap::new())
	}
}

impl<TBehavior> Default for Slots<TBehavior> {
	fn default() -> Self {
		Self::new()
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct Item<TBehavior> {
	pub slot: SlotKey,
	pub model: Option<Cow<'static, str>>,
	pub get_behavior: GetBehaviorFn<TBehavior>,
}

#[derive(Component, Debug, PartialEq)]
pub struct Equip<TBehavior>(pub Vec<Item<TBehavior>>);

impl<TBehavior> Equip<TBehavior> {
	pub fn new<const N: usize>(items: [Item<TBehavior>; N]) -> Self {
		Self(items.into())
	}
}
