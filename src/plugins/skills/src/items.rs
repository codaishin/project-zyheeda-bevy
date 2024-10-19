pub mod inventory_key;
pub mod slot_key;

use crate::skills::Skill;
use bevy::prelude::default;
use common::traits::accessors::get::Getter;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result};

#[derive(Debug, PartialEq, Clone, Copy, Default, Eq, Hash)]
pub enum Visualization {
	#[default]
	MountHand,
	MountForearm,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Item<TSkill = Skill> {
	pub name: &'static str,
	pub model: Option<&'static str>,
	pub skill: Option<TSkill>,
	pub item_type: ItemType,
}

impl<TSkill> Default for Item<TSkill> {
	fn default() -> Self {
		Self {
			name: default(),
			model: default(),
			skill: default(),
			item_type: default(),
		}
	}
}

impl<TSkill> Display for Item<TSkill> {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "Item({})", self.name)
	}
}

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum ItemType {
	#[default]
	Pistol,
	Bracer,
}

impl Getter<Visualization> for ItemType {
	fn get(&self) -> Visualization {
		match self {
			ItemType::Pistol => Visualization::MountHand,
			ItemType::Bracer => Visualization::MountForearm,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn item_type_mounts() {
		assert_eq!(
			[Visualization::MountHand, Visualization::MountForearm],
			[ItemType::Pistol.get(), ItemType::Bracer.get()]
		);
	}
}
