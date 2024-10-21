use super::visualization::Visualization;
use common::traits::accessors::get::Getter;
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
