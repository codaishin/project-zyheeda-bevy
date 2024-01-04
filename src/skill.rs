use crate::{
	behaviors::meta::BehaviorMeta,
	components::{ItemType, SlotKey},
	markers::meta::MarkerMeta,
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

#[derive(PartialEq, Debug, Clone, Default)]
pub struct Skill<TData = ()> {
	pub name: &'static str,
	pub data: TData,
	pub cast: Cast,
	pub soft_override: bool,
	pub marker: MarkerMeta,
	pub behavior: BehaviorMeta,
	pub is_usable_with: HashSet<ItemType>,
}

impl<T> Display for Skill<T> {
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

#[derive(Default, Debug, PartialEq, Clone)]
pub struct Queued {
	pub ray: Ray,
	pub slot_key: SlotKey,
}

#[derive(PartialEq, Debug, Clone, Default)]
pub struct Active {
	pub ray: Ray,
	pub slot_key: SlotKey,
}

impl Skill {
	pub fn with<T: Clone>(self, data: &T) -> Skill<T> {
		Skill {
			data: data.clone(),
			name: self.name,
			cast: self.cast,
			soft_override: self.soft_override,
			marker: self.marker,
			behavior: self.behavior,
			is_usable_with: self.is_usable_with,
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
			is_usable_with: self.is_usable_with,
		}
	}
}

impl Skill<Queued> {
	pub fn to_active(self) -> Skill<Active> {
		self.map_data(|queued| Active {
			ray: queued.ray,
			slot_key: queued.slot_key,
		})
	}
}
