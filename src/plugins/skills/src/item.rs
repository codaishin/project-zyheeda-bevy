pub mod item_type;

use crate::{
	components::renderer::{EssenceRender, ModelRender, Renderer},
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
		matches!(self.render.model, ModelRender::Hand(_))
	}
}

impl<TAgent> UsesView<ForearmSlots<TAgent>> for SkillItemContent {
	fn uses_view(&self) -> bool {
		matches!(self.render.model, ModelRender::Forearm(_))
	}
}

impl<TAgent> UsesView<SubModels<TAgent>> for SkillItemContent {
	fn uses_view(&self) -> bool {
		matches!(self.render.essence, EssenceRender::Material(_))
	}
}

impl Getter<AssetModel> for SkillItemContent {
	fn get(&self) -> AssetModel {
		match self.render.model {
			ModelRender::Hand(model) => model,
			ModelRender::Forearm(model) => model,
			ModelRender::None => AssetModel::None,
		}
	}
}

impl Getter<EssenceRender> for SkillItemContent {
	fn get(&self) -> EssenceRender {
		self.render.essence.clone()
	}
}
