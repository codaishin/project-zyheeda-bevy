pub mod item_type;

pub(crate) mod dto;

use crate::{
	components::renderer::{ModelRender, Renderer},
	definitions::{
		item_slots::{ForearmSlots, HandSlots},
		sub_models::SubModels,
	},
	skills::Skill,
	slot_key::SlotKey,
};
use bevy::prelude::*;
use common::components::AssetModel;
use item_type::SkillItemType;
use items::{
	item::Item,
	traits::{get_view_data::GetViewData, view::ItemView},
};
use player::components::player::Player;

pub type SkillItem = Item<SkillItemContent>;

#[derive(Debug, PartialEq, Default, Clone, TypePath)]
pub struct SkillItemContent {
	pub render: Renderer,
	pub skill: Option<Handle<Skill>>,
	pub item_type: SkillItemType,
}

impl GetViewData<HandSlots<Player>, SlotKey> for SkillItemContent {
	fn get_view_data(&self) -> <HandSlots<Player> as ItemView<SlotKey>>::TViewComponents {
		match &self.render.model {
			ModelRender::Hand(model) => model.clone(),
			_ => AssetModel::None,
		}
	}
}
impl GetViewData<ForearmSlots<Player>, SlotKey> for SkillItemContent {
	fn get_view_data(&self) -> <ForearmSlots<Player> as ItemView<SlotKey>>::TViewComponents {
		match &self.render.model {
			ModelRender::Forearm(model) => model.clone(),
			_ => AssetModel::None,
		}
	}
}
impl GetViewData<SubModels<Player>, SlotKey> for SkillItemContent {
	fn get_view_data(&self) -> <SubModels<Player> as ItemView<SlotKey>>::TViewComponents {
		self.render.essence.clone()
	}
}
