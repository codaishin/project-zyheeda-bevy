pub(crate) mod dto;

use crate::{
	components::model_render::ModelRender,
	skills::Skill,
	traits::visualize_item::VisualizeItem,
};
use bevy::prelude::*;
use common::{
	components::{asset_model::AssetModel, essence::Essence},
	tools::{item_type::ItemType, skill_execution::SkillExecution},
	traits::{
		accessors::get::RefInto,
		handles_loadout::loadout::{ItemToken, NoSkill, SkillIcon, SkillToken},
		handles_localization::Token,
		visible_slots::{EssenceSlot, ForearmSlot, HandSlot},
	},
};

#[derive(Debug, PartialEq, Default, Clone, Asset, TypePath)]
pub struct Item {
	pub token: Token,
	pub model: ModelRender,
	pub essence: Essence,
	pub skill: Option<Handle<Skill>>,
	pub item_type: ItemType,
}

impl<'a> RefInto<'a, &'a Token> for Item {
	fn ref_into(&self) -> &Token {
		&self.token
	}
}

impl From<&Item> for ItemType {
	fn from(Item { item_type, .. }: &Item) -> Self {
		*item_type
	}
}

impl<'a> From<&'a Item> for Option<&'a Handle<Skill>> {
	fn from(Item { skill, .. }: &'a Item) -> Self {
		skill.as_ref()
	}
}

impl VisualizeItem for EssenceSlot {
	type TComponent = Essence;

	fn visualize(item: Option<&Item>) -> Self::TComponent {
		match item {
			Some(Item { essence, .. }) => *essence,
			_ => Essence::None,
		}
	}
}

impl VisualizeItem for ForearmSlot {
	type TComponent = AssetModel;

	fn visualize(item: Option<&Item>) -> Self::TComponent {
		match item {
			Some(Item {
				model: ModelRender::Forearm(path),
				..
			}) => AssetModel::path(path),
			_ => AssetModel::none(),
		}
	}
}

impl VisualizeItem for HandSlot {
	type TComponent = AssetModel;

	fn visualize(item: Option<&Item>) -> Self::TComponent {
		match item {
			Some(Item {
				model: ModelRender::Hand(path),
				..
			}) => AssetModel::path(path),
			_ => AssetModel::none(),
		}
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct SkillItem {
	pub(crate) token: Token,
	pub(crate) skill: Option<ItemSkill>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ItemSkill {
	pub(crate) token: Token,
	pub(crate) icon: Handle<Image>,
	pub(crate) execution: SkillExecution,
}

impl<'a> From<&'a SkillItem> for ItemToken<'a> {
	fn from(item: &'a SkillItem) -> Self {
		ItemToken(&item.token)
	}
}

impl<'a> From<&'a SkillItem> for Result<SkillToken<'a>, NoSkill> {
	fn from(item: &'a SkillItem) -> Self {
		match &item.skill {
			Some(ItemSkill { token, .. }) => Ok(SkillToken(token)),
			_ => Err(NoSkill),
		}
	}
}

impl<'a> From<&'a SkillItem> for Result<SkillIcon<'a>, NoSkill> {
	fn from(item: &'a SkillItem) -> Self {
		match &item.skill {
			Some(ItemSkill { icon, .. }) => Ok(SkillIcon(icon)),
			_ => Err(NoSkill),
		}
	}
}

impl<'a> From<&'a SkillItem> for Result<&'a SkillExecution, NoSkill> {
	fn from(item: &'a SkillItem) -> Self {
		match &item.skill {
			Some(ItemSkill { execution, .. }) => Ok(execution),
			_ => Err(NoSkill),
		}
	}
}
