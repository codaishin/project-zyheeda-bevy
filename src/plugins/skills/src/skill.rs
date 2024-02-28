use crate::components::{Handed, ItemType, SideUnset, SlotKey};
use bevy::{
	ecs::system::EntityCommands,
	math::Ray,
	transform::components::{GlobalTransform, Transform},
};
use common::{components::Outdated, resources::ColliderInfo};
use std::{
	collections::{HashMap, HashSet},
	fmt::{Display, Formatter, Result},
	time::Duration,
};

#[derive(PartialEq, Debug, Clone)]
pub struct Skill<TAnimationKey = PlayerSkills<SideUnset>, TData = ()> {
	pub name: &'static str,
	pub data: TData,
	pub cast: Cast,
	pub soft_override: bool,
	pub animate: TAnimationKey,
	pub execution: SkillExecution,
	pub is_usable_with: HashSet<ItemType>,
}

impl<TAnimationKey: Default, TData: Default> Default for Skill<TAnimationKey, TData> {
	fn default() -> Self {
		Self {
			name: Default::default(),
			data: Default::default(),
			cast: Default::default(),
			soft_override: Default::default(),
			animate: Default::default(),
			execution: Default::default(),
			is_usable_with: Default::default(),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlayerSkills<TSide> {
	#[default]
	Idle,
	Shoot(Handed<TSide>),
	SwordStrike(TSide),
}

impl<TAnimationKey, TData> Display for Skill<TAnimationKey, TData> {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		match self.name {
			"" => write!(f, "Skill(<no name>)"),
			name => write!(f, "Skill({})", name),
		}
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct SelectInfo<T> {
	pub ray: Ray,
	pub collision_info: Option<ColliderInfo<T>>,
}

impl<T> Default for SelectInfo<T> {
	fn default() -> Self {
		Self {
			ray: Ray::default(),
			collision_info: None,
		}
	}
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Queued {
	pub target: Target,
	pub slot_key: SlotKey,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Active {
	pub target: Target,
	pub slot_key: SlotKey,
}

impl Skill {
	pub fn with<TData: Clone>(self, data: &TData) -> Skill<PlayerSkills<SideUnset>, TData> {
		Skill {
			data: data.clone(),
			name: self.name,
			cast: self.cast,
			soft_override: self.soft_override,
			animate: self.animate,
			execution: self.execution,
			is_usable_with: self.is_usable_with,
		}
	}
}

impl<TAnimationKey, TSrc> Skill<TAnimationKey, TSrc> {
	pub fn map_data<TDst>(self, map: fn(TSrc) -> TDst) -> Skill<TAnimationKey, TDst> {
		Skill {
			name: self.name,
			data: map(self.data),
			cast: self.cast,
			animate: self.animate,
			soft_override: self.soft_override,
			execution: self.execution,
			is_usable_with: self.is_usable_with,
		}
	}
}

impl<TAnimationKey> Skill<TAnimationKey, Queued> {
	pub fn to_active(self) -> Skill<TAnimationKey, Active> {
		self.map_data(|queued| Active {
			target: queued.target,
			slot_key: queued.slot_key,
		})
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
