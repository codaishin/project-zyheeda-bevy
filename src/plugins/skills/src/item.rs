pub mod item_type;

use crate::{
	components::renderer::{EssenceRender, Renderer},
	definitions::{
		item_slots::{ForearmSlots, HandSlots},
		sub_models::SubModels,
	},
	skills::Skill,
};
use common::{components::AssetModel, traits::accessors::get::Getter};
use item_type::SkillItemType;
use items::{item::Item, traits::uses_view::UsesView};

pub type SkillItem<TSkill = Skill> = Item<SkillItemContent<TSkill>>;

#[derive(Debug, PartialEq, Default, Clone)]
pub struct SkillItemContent<TSkill = Skill> {
	pub render: Renderer,
	pub skill: Option<TSkill>,
	pub item_type: SkillItemType,
}

impl<TAgent> UsesView<HandSlots<TAgent>> for SkillItemContent {
	fn uses_view(&self) -> bool {
		match self.item_type {
			SkillItemType::Pistol => true,
			SkillItemType::Bracer => false,
			SkillItemType::Essence => false,
		}
	}
}

impl<TAgent> UsesView<ForearmSlots<TAgent>> for SkillItemContent {
	fn uses_view(&self) -> bool {
		match self.item_type {
			SkillItemType::Pistol => false,
			SkillItemType::Bracer => true,
			SkillItemType::Essence => false,
		}
	}
}

impl<TAgent> UsesView<SubModels<TAgent>> for SkillItemContent {
	fn uses_view(&self) -> bool {
		match self.item_type {
			SkillItemType::Pistol => false,
			SkillItemType::Bracer => false,
			SkillItemType::Essence => true,
		}
	}
}

impl Getter<AssetModel> for SkillItemContent {
	fn get(&self) -> AssetModel {
		self.render.model
	}
}

impl Getter<EssenceRender> for SkillItemContent {
	fn get(&self) -> EssenceRender {
		self.render.essence.clone()
	}
}
