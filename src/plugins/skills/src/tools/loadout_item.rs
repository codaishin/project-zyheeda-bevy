use bevy::prelude::*;
use common::{
	tools::{item_description::ItemToken, skill_icon::SkillIcon},
	traits::{handles_localization::Token, inspect_able::InspectAble},
};

#[derive(Debug, PartialEq)]
pub struct LoadoutItem {
	pub token: Token,
	pub skill_icon: Option<Handle<Image>>,
}

impl InspectAble<ItemToken> for LoadoutItem {
	fn get_inspect_able_field(&self) -> &Token {
		&self.token
	}
}

impl InspectAble<SkillIcon> for LoadoutItem {
	fn get_inspect_able_field(&self) -> &Option<Handle<Image>> {
		&self.skill_icon
	}
}
