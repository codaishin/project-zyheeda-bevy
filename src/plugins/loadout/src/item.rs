pub(crate) mod dto;

use crate::{
	components::model_render::ModelRender,
	skills::Skill,
	traits::visualize_item::VisualizeItem,
};
use bevy::prelude::*;
use common::{components::essence::Essence, prelude::*, tools::path::Path};

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

impl AssetFolderPath for Item {
	fn asset_folder_path() -> Path {
		Path::from("items")
	}
}

impl VisualizeItem for VisibleEssenceSlot {
	type TComponent = Essence;

	fn visualize(item: Option<&Item>) -> Self::TComponent {
		match item {
			Some(Item { essence, .. }) => *essence,
			_ => Essence::None,
		}
	}
}

impl VisualizeItem for VisibleForearmSlot {
	type TComponent = Model;

	fn visualize(item: Option<&Item>) -> Self::TComponent {
		match item {
			Some(Item {
				model: ModelRender::Forearm(path),
				..
			}) => Model::scene(path),
			_ => Model::None,
		}
	}
}

impl VisualizeItem for VisibleHandSlot {
	type TComponent = Model;

	fn visualize(item: Option<&Item>) -> Self::TComponent {
		match item {
			Some(Item {
				model: ModelRender::Hand(path),
				..
			}) => Model::scene(path),
			_ => Model::None,
		}
	}
}
