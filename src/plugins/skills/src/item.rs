pub(crate) mod dto;

use crate::{components::model_render::ModelRender, skills::Skill};
use bevy::prelude::*;
use common::{
	components::essence::Essence,
	tools::{item_description::ItemToken, item_type::ItemType},
	traits::{handles_localization::Token, inspect_able::InspectAble},
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

impl From<&Item> for ItemType {
	fn from(Item { item_type, .. }: &Item) -> Self {
		*item_type
	}
}

impl<'a> From<&'a Item> for Option<&'a Handle<Skill>> {
	fn from(Item { skill, .. }: &'a Item) -> Self {
		skill.as_ref()
	}
}
