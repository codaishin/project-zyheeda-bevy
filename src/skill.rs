use crate::{
	behaviors::meta::BehaviorMeta,
	components::{ItemType, SlotKey},
	markers::meta::MarkerMeta,
};
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
pub struct SkillComboTree {
	pub skill: Skill,
	pub next: HashMap<SlotKey, SkillComboTree>,
}
