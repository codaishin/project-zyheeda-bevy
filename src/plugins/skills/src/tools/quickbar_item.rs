use bevy::prelude::*;
use common::{
	tools::skill_execution::SkillExecution,
	traits::{accessors::get::RefInto, handles_localization::Token},
};

#[derive(Debug, PartialEq, Default)]
pub struct QuickbarItem {
	pub skill_token: Token,
	pub skill_icon: Option<Handle<Image>>,
	pub execution: SkillExecution,
}

impl<'a> RefInto<'a, &'a Token> for QuickbarItem {
	fn ref_into(&self) -> &Token {
		&self.skill_token
	}
}

impl<'a> RefInto<'a, &'a Option<Handle<Image>>> for QuickbarItem {
	fn ref_into(&'a self) -> &'a Option<Handle<Image>> {
		&self.skill_icon
	}
}

impl<'a> RefInto<'a, &'a SkillExecution> for QuickbarItem {
	fn ref_into(&'a self) -> &'a SkillExecution {
		&self.execution
	}
}
