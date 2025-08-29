use super::combo_overview::ComboOverview;
use crate::{Tooltip, traits::insert_ui_content::InsertUiContent};
use bevy::{ecs::relationship::RelatedSpawnerCommands, prelude::*};
use common::{
	tools::{action_key::slot::PlayerSlot, skill_description::SkillToken, skill_icon::SkillIcon},
	traits::{
		handles_localization::LocalizeToken,
		inspect_able::{InspectAble, InspectField},
		thread_safe::ThreadSafe,
	},
};
use std::marker::PhantomData;

#[derive(Debug, Default, PartialEq, Clone)]
pub(crate) struct DropdownTrigger;

#[derive(Debug, Default, PartialEq, Clone)]
pub(crate) struct Vertical;

#[derive(Debug, Default, PartialEq, Clone)]
pub(crate) struct Horizontal;

#[derive(Debug, Default, PartialEq, Clone)]
pub(crate) struct DropdownItem<TLayout>(PhantomData<TLayout>);

#[derive(Component, Debug, Default, PartialEq, Clone)]
#[require(Node)]
pub(crate) struct ComboSkillButton<T, TSkill> {
	phantom_data: PhantomData<T>,
	pub(crate) skill: TSkill,
	pub(crate) key_path: Vec<PlayerSlot>,
}

impl<T, TSkill> ComboSkillButton<T, TSkill> {
	pub(crate) fn new(skill: TSkill, key_path: Vec<PlayerSlot>) -> ComboSkillButton<T, TSkill> {
		ComboSkillButton {
			phantom_data: PhantomData,
			skill,
			key_path,
		}
	}
}

impl<T, TSkill> InsertUiContent for ComboSkillButton<T, TSkill>
where
	T: Clone + ThreadSafe,
	TSkill: InspectAble<SkillToken> + InspectAble<SkillIcon> + Clone + ThreadSafe,
{
	fn insert_ui_content<TLocalization>(
		&self,
		localize: &TLocalization,
		parent: &mut RelatedSpawnerCommands<ChildOf>,
	) where
		TLocalization: LocalizeToken,
	{
		let token = SkillToken::inspect_field(&self.skill);
		let name = localize.localize_token(token.clone()).or_token();

		parent.spawn((
			self.clone(),
			ComboOverview::skill_button(SkillIcon::inspect_field(&self.skill).clone()),
			Name::from(name.clone()),
			Tooltip::new(name),
		));
	}
}
