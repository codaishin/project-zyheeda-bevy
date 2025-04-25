use bevy::prelude::*;
use common::{
	tools::{
		skill_description::SkillToken,
		skill_execution::SkillExecution,
		skill_icon::SkillIcon,
	},
	traits::{handles_localization::Token, inspect_able::InspectAble},
};

#[derive(Debug, PartialEq, Default)]
pub struct QuickbarItem {
	pub skill_token: Token,
	pub skill_icon: Option<Handle<Image>>,
	pub execution: SkillExecution,
}

impl InspectAble<SkillToken> for QuickbarItem {
	fn get_inspect_able_field(&self) -> &Token {
		&self.skill_token
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
