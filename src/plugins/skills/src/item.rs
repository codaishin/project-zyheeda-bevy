pub mod item_type;

use crate::{
	components::renderer::{ModelRender, Renderer},
	definitions::{
		item_slots::{ForearmSlots, HandSlots},
		sub_models::SubModels,
	},
	skills::Skill,
	slot_key::SlotKey,
};
use common::components::{AssetModel, Player};
use item_type::SkillItemType;
use items::{
	item::Item,
	traits::{get_view_data::GetViewData, view::ItemView},
};

pub type SkillItem<TSkill = Skill> = Item<SkillItemContent<TSkill>>;

#[derive(Debug, PartialEq, Default, Clone)]
pub struct SkillItemContent<TSkill = Skill> {
	pub render: Renderer,
	pub skill: Option<TSkill>,
	pub item_type: SkillItemType,
}

impl GetViewData<HandSlots<Player>, SlotKey> for SkillItemContent {
	fn get_view_data(&self) -> <HandSlots<Player> as ItemView<SlotKey>>::TViewComponents {
		match self.render.model {
			ModelRender::Hand(model) => model,
			_ => AssetModel::None,
		}
	}
}
impl GetViewData<ForearmSlots<Player>, SlotKey> for SkillItemContent {
	fn get_view_data(&self) -> <ForearmSlots<Player> as ItemView<SlotKey>>::TViewComponents {
		match self.render.model {
			ModelRender::Forearm(model) => model,
			_ => AssetModel::None,
		}
	}
}
impl GetViewData<SubModels<Player>, SlotKey> for SkillItemContent {
	fn get_view_data(&self) -> <SubModels<Player> as ItemView<SlotKey>>::TViewComponents {
		self.render.essence.clone()
	}
}
