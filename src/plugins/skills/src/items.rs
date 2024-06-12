pub mod inventory_key;
pub mod slot_key;

use crate::skills::Skill;
use std::{
	collections::HashSet,
	fmt::{Display, Formatter, Result},
};

#[derive(Debug, PartialEq, Clone, Copy, Default, Eq, Hash)]
pub enum Mount {
	#[default]
	Hand,
	Forearm,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Item {
	pub name: &'static str,
	pub model: Option<&'static str>,
	pub skill: Option<Skill>,
	pub item_type: HashSet<ItemType>,
	pub mount: Mount,
}

impl Display for Item {
	fn fmt(&self, f: &mut Formatter) -> Result {
		match &self.skill {
			Some(skill) => write!(f, "Item({}, {})", self.name, skill),
			None => write!(f, "Item({}, <no skill>)", self.name),
		}
	}
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum ItemType {
	Pistol,
	Bracer,
}
