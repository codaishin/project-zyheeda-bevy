use crate::{
	behaviors::meta::BehaviorMeta,
	components::{ItemType, PlayerSkills, SideUnset, SlotKey},
	resources::MouseHover,
};
use bevy::math::Ray;
use std::{
	collections::{HashMap, HashSet},
	fmt::{Display, Formatter, Result},
	time::Duration,
};

#[derive(PartialEq, Debug, Clone, Copy, Default)]
pub struct Cast {
	pub pre: Duration,
	pub active: Duration,
	pub after: Duration,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Skill<TAnimationKey = PlayerSkills<SideUnset>, TData = ()> {
	pub name: &'static str,
	pub data: TData,
	pub cast: Cast,
	pub soft_override: bool,
	pub animate: TAnimationKey,
	pub behavior: BehaviorMeta,
	pub is_usable_with: HashSet<ItemType>,
}

impl<TData: Default> Default for Skill<PlayerSkills<SideUnset>, TData> {
	fn default() -> Self {
		Self {
			name: Default::default(),
			data: Default::default(),
			cast: Default::default(),
			soft_override: Default::default(),
			animate: Default::default(),
			behavior: Default::default(),
			is_usable_with: Default::default(),
		}
	}
}

impl<TAnimationKey, TData> Display for Skill<TAnimationKey, TData> {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		match self.name {
			"" => write!(f, "Skill(<no name>)"),
			name => write!(f, "Skill({})", name),
		}
	}
}

#[derive(PartialEq, Debug, Clone)]
pub enum SkillComboNext {
	Tree(HashMap<SlotKey, SkillComboTree<SkillComboNext>>),
	Alternate { slot_key: SlotKey, skill: Skill },
}

impl SkillComboNext {
	pub fn done() -> SkillComboNext {
		SkillComboNext::Tree(HashMap::new())
	}
}

#[derive(PartialEq, Debug, Clone)]
pub struct SkillComboTree<TNext> {
	pub skill: Skill,
	pub next: TNext,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct SelectInfo {
	pub ray: Ray,
	pub hover: MouseHover,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Queued {
	pub select_info: SelectInfo,
	pub slot_key: SlotKey,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Active {
	pub select_info: SelectInfo,
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
			behavior: self.behavior,
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
			behavior: self.behavior,
			is_usable_with: self.is_usable_with,
		}
	}
}

impl<TAnimationKey> Skill<TAnimationKey, Queued> {
	pub fn to_active(self) -> Skill<TAnimationKey, Active> {
		self.map_data(|queued| Active {
			select_info: queued.select_info,
			slot_key: queued.slot_key,
		})
	}
}

pub struct SwordStrike;
