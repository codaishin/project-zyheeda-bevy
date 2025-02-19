use bevy::prelude::*;
use common::{
	tools::{
		item_description::ItemDescription,
		skill_execution::SkillExecution,
		skill_icon::SkillIcon,
	},
	traits::inspect_able::InspectAble,
};

pub struct QuickbarItem {
	pub name: String,
	pub icon: Option<Handle<Image>>,
	pub execution: SkillExecution,
}

impl InspectAble<ItemDescription> for QuickbarItem {
	fn get_inspect_able_field(&self) -> String {
		self.name.clone()
	}
}

impl InspectAble<SkillIcon> for QuickbarItem {
	fn get_inspect_able_field(&self) -> &Option<Handle<Image>> {
		&self.icon
	}
}

impl InspectAble<SkillExecution> for QuickbarItem {
	fn get_inspect_able_field(&self) -> &SkillExecution {
		&self.execution
	}
}
