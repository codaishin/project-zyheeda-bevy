use crate::definitions::item_slots::{ForearmSlots, HandSlots};
use items::traits::uses_view::UsesView;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum SkillItemType {
	#[default]
	Pistol,
	Bracer,
}

impl<TAgent> UsesView<HandSlots<TAgent>> for SkillItemType {
	fn uses_view(&self) -> bool {
		match self {
			SkillItemType::Pistol => true,
			SkillItemType::Bracer => false,
		}
	}
}

impl<TAgent> UsesView<ForearmSlots<TAgent>> for SkillItemType {
	fn uses_view(&self) -> bool {
		match self {
			SkillItemType::Pistol => false,
			SkillItemType::Bracer => true,
		}
	}
}
