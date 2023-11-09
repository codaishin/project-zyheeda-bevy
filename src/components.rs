use crate::{behaviors::MovementMode, types::BoneName};
use bevy::{
	prelude::{Component, *},
	utils::HashMap,
};
use std::{borrow::Cow, collections::VecDeque, fmt::Debug, marker::PhantomData};

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
pub struct WaitNext<TBehavior> {
	phantom_data: PhantomData<TBehavior>,
}

impl<TBehavior> WaitNext<TBehavior> {
	pub fn new() -> Self {
		Self {
			phantom_data: PhantomData,
		}
	}
}

impl<TBehavior> Default for WaitNext<TBehavior> {
	fn default() -> Self {
		Self::new()
	}
}

pub struct Walk;

pub struct Run;

#[derive(Component)]
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

#[derive(Component, Clone, Copy, PartialEq, Debug)]
pub struct SimpleMovement<TBehavior> {
	pub target: Vec3,
	phantom_data: PhantomData<TBehavior>,
}

impl<TBehavior> SimpleMovement<TBehavior> {
	pub fn new(target: Vec3) -> Self {
		Self {
			target,
			phantom_data: PhantomData,
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
	Hand(Side),
	Legs,
}

#[derive(Component, Clone, PartialEq, Debug)]
pub struct SlotBones(pub HashMap<SlotKey, Cow<'static, BoneName>>);

impl SlotBones {
	pub fn new<const C: usize>(pairs: [(SlotKey, &'static BoneName); C]) -> Self {
		Self(pairs.map(|(k, v)| (k, Cow::from(v))).into())
	}
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum ScheduleMode {
	Enqueue,
	Override,
}

pub type GetBehaviorFn<TBehavior> = fn(ray: Ray) -> Option<TBehavior>;

#[derive(Component, PartialEq, Debug)]
pub struct Schedule<TBehavior> {
	pub mode: ScheduleMode,
	pub get_behaviors: Vec<GetBehaviorFn<TBehavior>>,
}

#[derive(PartialEq, Debug)]
pub struct Slot<TBehavior> {
	pub entity: Entity,
	pub get_behavior: Option<GetBehaviorFn<TBehavior>>,
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
	pub get_behavior: Option<GetBehaviorFn<TBehavior>>,
}

#[derive(Component, Debug, PartialEq)]
pub struct Equip<TBehavior>(pub Vec<Item<TBehavior>>);

impl<TBehavior> Equip<TBehavior> {
	pub fn new<const N: usize>(items: [Item<TBehavior>; N]) -> Self {
		Self(items.into())
	}
}

#[derive(Component)]
pub struct Queue<T>(pub VecDeque<T>);
