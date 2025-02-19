use bevy::prelude::*;
use common::{
	tools::{item_description::ItemDescription, skill_icon::SkillIcon},
	traits::inspect_able::InspectAble,
};

#[derive(Debug, PartialEq)]
pub struct LoadoutItem {
	pub name: String,
	pub skill_icon: Option<Handle<Image>>,
}

impl InspectAble<ItemDescription> for LoadoutItem {
	fn get_inspect_able_field(&self) -> String {
		self.name.clone()
	}
}

impl InspectAble<SkillIcon> for LoadoutItem {
	fn get_inspect_able_field(&self) -> &Option<Handle<Image>> {
		&self.skill_icon
	}
}
