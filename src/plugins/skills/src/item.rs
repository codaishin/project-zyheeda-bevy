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
		accessors::get::GetProperty,
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

impl GetProperty<ItemType> for Item {
	fn get_property(&self) -> ItemType {
		self.item_type
	}
}

impl GetProperty<Option<Handle<Skill>>> for Item {
	fn get_property(&self) -> Option<&'_ Handle<Skill>> {
		self.skill.as_ref()
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

impl GetProperty<ItemToken> for SkillItem {
	fn get_property(&self) -> &Token {
		&self.token
	}
}
impl GetProperty<Result<SkillToken, NoSkill>> for SkillItem {
	fn get_property(&self) -> Result<&'_ Token, NoSkill> {
		match &self.skill {
			Some(ItemSkill { token, .. }) => Ok(token),
			_ => Err(NoSkill),
		}
	}
}

impl GetProperty<Result<SkillIcon, NoSkill>> for SkillItem {
	fn get_property(&self) -> Result<&'_ Handle<Image>, NoSkill> {
		match &self.skill {
			Some(ItemSkill { icon, .. }) => Ok(icon),
			_ => Err(NoSkill),
		}
	}
}

impl GetProperty<Result<SkillExecution, NoSkill>> for SkillItem {
	fn get_property(&self) -> Result<SkillExecution, NoSkill> {
		match &self.skill {
			Some(ItemSkill { execution, .. }) => Ok(*execution),
			_ => Err(NoSkill),
		}
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct ItemSkill {
	pub(crate) token: Token,
	pub(crate) icon: Handle<Image>,
	pub(crate) execution: SkillExecution,
}
