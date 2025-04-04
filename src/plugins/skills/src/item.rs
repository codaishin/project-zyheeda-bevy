pub(crate) mod dto;

use crate::{components::model_render::ModelRender, skills::Skill};
use bevy::prelude::*;
use common::{
	components::essence::Essence,
	tools::{item_description::ItemDescription, item_type::ItemType},
	traits::{
		accessors::get::{Getter, GetterRef},
		inspect_able::InspectAble,
	},
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

impl InspectAble<ItemDescription> for Item {
	fn get_inspect_able_field(&self) -> String {
		self.name.clone()
	}
}

impl Getter<ItemType> for Item {
	fn get(&self) -> ItemType {
		self.item_type
	}
}

impl GetterRef<Option<Handle<Skill>>> for Item {
	fn get(&self) -> &Option<Handle<Skill>> {
		&self.skill
	}
}
