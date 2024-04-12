use crate::components::{Handed, ItemType, SideUnset, SlotKey};
use bevy::{
	ecs::system::EntityCommands,
	math::{primitives::Direction3d, Ray3d, Vec3},
	transform::components::{GlobalTransform, Transform},
};
use common::{components::Outdated, resources::ColliderInfo};
use std::{
	collections::{HashMap, HashSet},
	fmt::{Display, Formatter, Result},
	time::Duration,
};

#[derive(PartialEq, Debug, Clone)]
pub struct Skill<TData = ()> {
	pub name: &'static str,
	pub data: TData,
	pub cast: Cast,
	pub soft_override: bool,
	pub animate: Option<PlayerSkills<SideUnset>>,
	pub execution: SkillExecution,
	pub is_usable_with: HashSet<ItemType>,
	pub dual_wield: bool,
}

impl<TData: Default> Default for Skill<TData> {
	fn default() -> Self {
		Self {
			name: Default::default(),
			data: Default::default(),
			cast: Default::default(),
			soft_override: Default::default(),
			animate: Default::default(),
			execution: Default::default(),
			is_usable_with: Default::default(),
			dual_wield: Default::default(),
		}
	}
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Cast {
	pub aim: Duration,
	pub pre: Duration,
	pub active: Duration,
	pub after: Duration,
}

impl Default for Cast {
	fn default() -> Self {
		Self {
			aim: Duration::MAX,
			pre: Default::default(),
			active: Default::default(),
			after: Default::default(),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayerSkills<TSide> {
	Shoot(Handed<TSide>),
	SwordStrike(TSide),
}

impl<TData> Display for Skill<TData> {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		match self.name {
			"" => write!(f, "Skill(<no name>)"),
			name => write!(f, "Skill({})", name),
		}
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct SelectInfo<T> {
	pub ray: Ray3d,
	pub collision_info: Option<ColliderInfo<T>>,
}

impl<T> Default for SelectInfo<T> {
	fn default() -> Self {
		Self {
			ray: Ray3d {
				origin: Vec3::ZERO,
				direction: Direction3d::NEG_Z,
			},
			collision_info: None,
		}
	}
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Queued(pub SlotKey);

impl Skill {
	pub fn with<TData: Clone>(self, data: TData) -> Skill<TData> {
		Skill {
			data,
			name: self.name,
			cast: self.cast,
			soft_override: self.soft_override,
			animate: self.animate,
			execution: self.execution,
			is_usable_with: self.is_usable_with,
			dual_wield: self.dual_wield,
		}
	}
}

impl<TSrc> Skill<TSrc> {
	pub fn map_data<TDst>(self, map: fn(TSrc) -> TDst) -> Skill<TDst> {
		Skill {
			name: self.name,
			data: map(self.data),
			cast: self.cast,
			animate: self.animate,
			soft_override: self.soft_override,
			execution: self.execution,
			is_usable_with: self.is_usable_with,
			dual_wield: self.dual_wield,
		}
	}
}

pub(crate) struct SwordStrike;

#[derive(PartialEq, Debug, Clone, Copy, Eq, Hash)]
pub(crate) enum SkillState {
	Aim,
	PreCast,
	Active,
	AfterCast,
}

#[derive(PartialEq, Debug, Clone)]
pub(crate) struct SkillComboTree<TNext> {
	pub skill: Skill,
	pub next: TNext,
}

#[derive(PartialEq, Debug, Clone)]
pub(crate) enum SkillComboNext {
	Tree(HashMap<SlotKey, SkillComboTree<SkillComboNext>>),
	Alternate { slot_key: SlotKey, skill: Skill },
}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Spawner(pub GlobalTransform);

pub type Target = SelectInfo<Outdated<GlobalTransform>>;
pub type TransformFN = fn(&mut Transform, &Spawner, &Target);
pub type StartBehaviorFn = fn(&mut EntityCommands, &Transform, &Spawner, &Target);
pub type StopBehaviorFn = fn(&mut EntityCommands);

#[derive(PartialEq, Debug, Clone, Copy, Default)]
pub struct SkillExecution {
	pub run_fn: Option<StartBehaviorFn>,
	pub stop_fn: Option<StopBehaviorFn>,
}
