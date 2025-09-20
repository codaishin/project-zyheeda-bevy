use super::combo_overview::ComboOverview;
use crate::{components::label::UILabel, traits::insert_ui_content::InsertUiContent};
use bevy::{ecs::relationship::RelatedSpawnerCommands, prelude::*};
use common::{
	tools::action_key::slot::SlotKey,
	traits::{
		accessors::get::{DynProperty, GetProperty},
		handles_loadout::loadout::{SkillIcon, SkillToken},
		handles_localization::Localize,
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
	pub(crate) key_path: Vec<SlotKey>,
}

impl<T, TSkill> ComboSkillButton<T, TSkill> {
	pub(crate) fn new(skill: TSkill, key_path: Vec<SlotKey>) -> ComboSkillButton<T, TSkill> {
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
	TSkill: Clone + ThreadSafe + GetProperty<SkillToken> + GetProperty<SkillIcon>,
{
	fn insert_ui_content<TLocalization>(
		&self,
		_: &TLocalization,
		parent: &mut RelatedSpawnerCommands<ChildOf>,
	) where
		TLocalization: Localize,
	{
		parent.spawn((
			self.clone(),
			ComboOverview::skill_button(self.skill.dyn_property::<SkillIcon>().clone()),
			UILabel(self.skill.dyn_property::<SkillToken>().clone()),
		));
	}
}
