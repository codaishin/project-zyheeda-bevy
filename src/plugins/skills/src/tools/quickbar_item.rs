use bevy::prelude::*;
use common::{
	tools::{
		skill_description::SkillDescription,
		skill_execution::SkillExecution,
		skill_icon::SkillIcon,
	},
	traits::inspect_able::InspectAble,
};

#[derive(Debug, PartialEq, Default)]
pub struct QuickbarItem {
	pub skill_name: String,
	pub skill_icon: Option<Handle<Image>>,
	pub execution: SkillExecution,
}

impl InspectAble<SkillDescription> for QuickbarItem {
	fn get_inspect_able_field(&self) -> String {
		self.skill_name.clone()
	}
}

impl InspectAble<SkillIcon> for QuickbarItem {
	fn get_inspect_able_field(&self) -> &Option<Handle<Image>> {
		&self.skill_icon
	}
}

impl InspectAble<SkillExecution> for QuickbarItem {
	fn get_inspect_able_field(&self) -> &SkillExecution {
		&self.execution
	}
}
