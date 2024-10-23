pub mod item_type;

use crate::{
	definitions::item_slots::{ForearmSlots, HandSlots},
	skills::Skill,
};
use item_type::SkillItemType;
use items::{item::Item, traits::uses_view::UsesView};

pub type SkillItem<TSkill = Skill> = Item<SkillItemContent<TSkill>>;

#[derive(Debug, PartialEq, Default, Clone)]
pub struct SkillItemContent<TSkill> {
	pub skill: Option<TSkill>,
	pub item_type: SkillItemType,
}

impl<TAgent> UsesView<HandSlots<TAgent>> for SkillItem {
	fn uses_view(&self) -> bool {
		match self.content.item_type {
			SkillItemType::Pistol => true,
			SkillItemType::Bracer => false,
		}
	}
}

impl<TAgent> UsesView<ForearmSlots<TAgent>> for SkillItem {
	fn uses_view(&self) -> bool {
		match self.content.item_type {
			SkillItemType::Pistol => false,
			SkillItemType::Bracer => true,
		}
	}
}
