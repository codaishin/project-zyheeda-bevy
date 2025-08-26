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
		handles_loadout::{ItemToken, SkillIcon, SkillToken},
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
pub enum SkillItem {
	Some {
		item_token: Token,
		skill_token: Option<Token>,
		skill_icon: Option<Handle<Image>>,
		execution: SkillExecution,
	},
	None,
}

impl<'a> RefInto<'a, ItemToken<'a>> for SkillItem {
	fn ref_into(&'a self) -> ItemToken<'a> {
		match self {
			SkillItem::Some { item_token, .. } => ItemToken(Some(item_token)),
			SkillItem::None => ItemToken(None),
		}
	}
}

impl<'a> RefInto<'a, SkillToken<'a>> for SkillItem {
	fn ref_into(&'a self) -> SkillToken<'a> {
		match self {
			SkillItem::Some { skill_token, .. } => SkillToken(skill_token.as_ref()),
			SkillItem::None => SkillToken(None),
		}
	}
}

impl<'a> RefInto<'a, SkillIcon<'a>> for SkillItem {
	fn ref_into(&'a self) -> SkillIcon<'a> {
		match self {
			Self::Some { skill_icon, .. } => SkillIcon(skill_icon.as_ref()),
			_ => SkillIcon(None),
		}
	}
}

impl<'a> RefInto<'a, &'a SkillExecution> for SkillItem {
	fn ref_into(&'a self) -> &'a SkillExecution {
		match self {
			SkillItem::Some { execution, .. } => execution,
			SkillItem::None => &SkillExecution::None,
		}
	}
}
