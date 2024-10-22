use crate::definitions::item_slots::{ForearmSlots, HandSlots};
use items::traits::uses_visualizer::UsesVisualizer;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum SkillItemType {
	#[default]
	Pistol,
	Bracer,
}

impl<TAgent> UsesVisualizer<HandSlots<TAgent>> for SkillItemType {
	fn uses_visualization(&self) -> bool {
		match self {
			SkillItemType::Pistol => true,
			SkillItemType::Bracer => false,
		}
	}
}

impl<TAgent> UsesVisualizer<ForearmSlots<TAgent>> for SkillItemType {
	fn uses_visualization(&self) -> bool {
		match self {
			SkillItemType::Pistol => false,
			SkillItemType::Bracer => true,
		}
	}
}
