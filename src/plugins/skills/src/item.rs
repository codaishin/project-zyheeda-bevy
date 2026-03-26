pub(crate) mod dto;

use crate::{
	components::model_render::ModelRender,
	skills::Skill,
	traits::visualize_item::VisualizeItem,
};
use bevy::prelude::*;
use common::{
	components::{asset_model::AssetModel, essence::Essence},
	tools::item_type::ItemType,
	traits::{
		accessors::get::View,
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

impl View<ItemType> for Item {
	fn view(&self) -> ItemType {
		self.item_type
	}
}

impl View<Option<Handle<Skill>>> for Item {
	fn view(&self) -> Option<&'_ Handle<Skill>> {
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
