pub(crate) mod dto;

use crate::{components::model_render::ModelRender, skills::Skill};
use bevy::prelude::*;
use common::{
	components::essence::Essence,
	tools::item_type::ItemType,
	traits::{accessors::get::Getter, handles_equipment::ItemName},
};

#[derive(Debug, PartialEq, Default, Clone, Asset, TypePath)]
pub struct Item {
	pub name: String,
	pub model: ModelRender,
	pub essence: Essence,
	pub skill: Option<Handle<Skill>>,
	pub item_type: ItemType,
}

impl Item {
	pub fn named(name: &str) -> Self {
		Self {
			name: name.to_owned(),
			..default()
		}
	}
}

impl Getter<ItemName> for Item {
	fn get(&self) -> ItemName {
		ItemName(self.name.clone())
	}
}

impl Getter<ItemType> for Item {
	fn get(&self) -> ItemType {
		self.item_type
	}
}
