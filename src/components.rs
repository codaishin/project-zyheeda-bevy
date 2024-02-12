use crate::types::BoneName;
use bevy::{
	math::Vec2,
	prelude::{Component, Entity, Vec3},
};
use common::{
	components::SlotKey,
	skill::{Skill, SkillComboTree},
	tools::UnitsPerSecond,
};
use std::{collections::HashMap, fmt::Debug, marker::PhantomData, time::Duration};

#[derive(Component)]
pub struct CamOrbit {
	pub center: Vec3,
	pub distance: f32,
	pub sensitivity: f32,
}

#[derive(Component, Clone)]
pub enum VoidSpherePart {
	Core,
	RingA(UnitsPerSecond),
	RingB(UnitsPerSecond),
}

#[derive(Component)]
pub struct Health {
	pub current: u8,
	pub max: u8,
}

impl Health {
	pub fn new(value: u8) -> Self {
		Self {
			current: value,
			max: value,
		}
	}
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

#[derive(Component, Clone, PartialEq, Debug)]
pub struct SlotBones(pub HashMap<SlotKey, &'static BoneName>);

#[derive(Component, PartialEq, Debug)]
pub enum Schedule {
	Enqueue((SlotKey, Skill)),
	Override((SlotKey, Skill)),
	StopAimAfter(Duration),
	UpdateTarget,
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

#[derive(Component, Clone)]
pub struct ComboTreeTemplate<TNext>(pub HashMap<SlotKey, SkillComboTree<TNext>>);

#[derive(Component, PartialEq, Debug)]
pub struct ComboTreeRunning<TNext>(pub HashMap<SlotKey, SkillComboTree<TNext>>);

#[derive(Component)]
pub struct Dummy;

#[derive(Component, PartialEq, Debug)]
pub struct ColliderRoot(pub Entity);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UI {
	pub background: Entity,
	pub foreground: Entity,
}

#[derive(Component)]
pub struct Bar<T> {
	pub position: Option<Vec2>,
	pub ui: Option<UI>,
	pub current: f32,
	pub max: f32,
	pub scale: f32,
	pub phantom_data: PhantomData<T>,
}

impl<T> Bar<T> {
	pub fn new(position: Option<Vec2>, current: f32, max: f32, scale: f32) -> Self {
		Self {
			position,
			ui: None,
			current,
			max,
			scale,
			phantom_data: PhantomData,
		}
	}
}
