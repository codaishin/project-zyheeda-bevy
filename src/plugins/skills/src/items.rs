pub mod inventory_key;
pub mod slot_key;

use bevy::asset::Handle;
use common::traits::load_asset::Path;
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

#[derive(Default, Debug, PartialEq, Clone)]
pub enum SkillHandle {
	#[default]
	None,
	Path(Path),
	Handle(Handle<Skill>),
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Item {
	pub name: &'static str,
	pub model: Option<&'static str>,
	pub skill: SkillHandle,
	pub item_type: HashSet<ItemType>,
	pub mount: Mount,
}

impl Display for Item {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "Item({}) with Skill({:?})", self.name, self.skill)
	}
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum ItemType {
	Pistol,
	Bracer,
}
