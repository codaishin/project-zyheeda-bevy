use super::combo_overview::ComboOverview;
use crate::{Tooltip, traits::insert_ui_content::InsertUiContent};
use bevy::{ecs::relationship::RelatedSpawnerCommands, prelude::*};
use common::{
	tools::action_key::slot::PlayerSlot,
	traits::{
		accessors::get::{RefAs, RefInto},
		handles_localization::{Localize, Token},
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
	TSkill: Clone
		+ ThreadSafe
		+ for<'a> RefInto<'a, &'a Token>
		+ for<'a> RefInto<'a, &'a Option<Handle<Image>>>,
{
	fn insert_ui_content<TLocalization>(
		&self,
		localize: &TLocalization,
		parent: &mut RelatedSpawnerCommands<ChildOf>,
	) where
		TLocalization: Localize,
	{
		let token = self.skill.ref_as::<&Token>();
		let icon = self.skill.ref_as::<&Option<Handle<Image>>>();
		let name = localize.localize(token).or_token();

		parent.spawn((
			self.clone(),
			ComboOverview::skill_button(icon.clone()),
			Name::from(name.clone()),
			Tooltip::new(name),
		));
	}
}
