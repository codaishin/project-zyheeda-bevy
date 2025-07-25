pub(crate) mod dto;

use crate::{components::model_render::ModelRender, skills::Skill};
use bevy::prelude::*;
use common::{
	components::essence::Essence,
	tools::{item_description::ItemToken, item_type::ItemType},
	traits::{
		accessors::get::{Getter, GetterRef},
		handles_localization::Token,
		inspect_able::InspectAble,
	},
};

#[derive(Debug, PartialEq, Default, Clone, Asset, TypePath)]
pub struct Item {
	pub token: Token,
	pub model: ModelRender,
	pub essence: Essence,
	pub skill: Option<Handle<Skill>>,
	pub item_type: ItemType,
}

impl InspectAble<ItemToken> for Item {
	fn get_inspect_able_field(&self) -> &Token {
		&self.token
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
