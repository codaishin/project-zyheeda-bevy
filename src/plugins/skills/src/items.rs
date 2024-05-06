use common::components::Side;

use crate::skills::Skill;
use std::{
	collections::HashSet,
	fmt::{Display, Formatter, Result},
};

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Item {
	pub name: &'static str,
	pub model: Option<&'static str>,
	pub skill: Option<Skill>,
	pub item_type: HashSet<ItemType>,
}

impl Display for Item {
	fn fmt(&self, f: &mut Formatter) -> Result {
		match &self.skill {
			Some(skill) => write!(f, "Item({}, {})", self.name, skill),
			None => write!(f, "Item({}, <no skill>)", self.name),
		}
	}
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct InventoryKey(pub usize);

#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug, Default)]
pub enum SlotKey {
	#[default]
	SkillSpawn,
	Hand(Side),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum ItemType {
	Pistol,
	Sword,
	Legs,
}
