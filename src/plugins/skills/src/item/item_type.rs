use crate::definitions::item_slots::{ForearmSlots, HandSlots};

use super::visualization::Visualization;
use common::traits::accessors::get::Getter;
use items::traits::uses_visualizer::UsesVisualizer;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum SkillItemType {
	#[default]
	Pistol,
	Bracer,
}

impl Getter<Visualization> for SkillItemType {
	fn get(&self) -> Visualization {
		match self {
			SkillItemType::Pistol => Visualization::MountHand,
			SkillItemType::Bracer => Visualization::MountForearm,
		}
	}
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn item_type_mounts() {
		assert_eq!(
			[Visualization::MountHand, Visualization::MountForearm],
			[SkillItemType::Pistol.get(), SkillItemType::Bracer.get()]
		);
	}
}
