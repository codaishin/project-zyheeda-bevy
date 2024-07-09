pub mod inventory_key;
pub mod slot_key;

use bevy::prelude::default;
use serde::{Deserialize, Serialize};
use std::{
	collections::HashSet,
	fmt::{Display, Formatter, Result},
};

use crate::skills::Skill;

#[derive(Debug, PartialEq, Clone, Copy, Default, Eq, Hash)]
pub enum Mount {
	#[default]
	Hand,
	Forearm,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Item<TSkill = Skill> {
	pub name: &'static str,
	pub model: Option<&'static str>,
	pub skill: Option<TSkill>,
	pub item_type: HashSet<ItemType>,
	pub mount: Mount,
}

impl<TSkill> Default for Item<TSkill> {
	fn default() -> Self {
		Self {
			name: default(),
			model: default(),
			skill: default(),
			item_type: default(),
			mount: default(),
		}
	}
}

impl<TSkill> Display for Item<TSkill> {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "Item({})", self.name)
	}
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum ItemType {
	Pistol,
	Bracer,
}
