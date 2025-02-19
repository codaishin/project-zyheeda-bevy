use bevy::prelude::*;
use common::{
	tools::{item_description::ItemDescription, skill_icon::SkillIcon},
	traits::inspect_able::InspectAble,
};

pub struct InventoryItem {
	pub name: String,
	pub skill_icon: Option<Handle<Image>>,
}

impl InspectAble<ItemDescription> for InventoryItem {
	fn get_inspect_able_field(&self) -> String {
		self.name.clone()
	}
}

impl InspectAble<SkillIcon> for InventoryItem {
	fn get_inspect_able_field(&self) -> &Option<Handle<Image>> {
		&self.skill_icon
	}
}
