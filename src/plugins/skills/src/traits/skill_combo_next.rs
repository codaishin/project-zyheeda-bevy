use super::ComboNext;
use crate::{
	components::SlotKey,
	skill::{Queued, Skill, SkillComboNext, SkillComboTree},
};

impl ComboNext for SkillComboNext {
	fn to_vec(&self, trigger_skill: &Skill<Queued>) -> Vec<(SlotKey, SkillComboTree<Self>)> {
		match &self {
			SkillComboNext::Tree(tree) => tree.clone().into_iter().collect(),
			SkillComboNext::Alternate { slot_key, skill } => {
				vec![(
					*slot_key,
					SkillComboTree {
						skill: skill.clone(),
						next: SkillComboNext::Alternate {
							slot_key: trigger_skill.data.slot_key,
							skill: trigger_skill.clone().map_data(|_| ()),
						},
					},
				)]
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::utils::default;
	use common::components::Side;
	use std::collections::HashMap;

	#[test]
	fn tree_to_branches() {
		let branches = [(
			SlotKey::Hand(Side::Main),
			SkillComboTree {
				skill: Skill {
					name: "my skill",
					..default()
				},
				next: SkillComboNext::Tree(HashMap::new()),
			},
		)];
		let next = SkillComboNext::Tree(HashMap::from(branches.clone()));
		assert_eq!(branches.to_vec(), next.to_vec(&default()));
	}

	#[test]
	fn alternate_to_branches() {
		let next = SkillComboNext::Alternate {
			slot_key: SlotKey::Hand(Side::Off),
			skill: Skill {
				name: "alternate skill",
				..default()
			},
		};
		let skill = Skill {
			name: "skill",
			data: Queued {
				slot_key: SlotKey::Hand(Side::Main),
				..default()
			},
			..default()
		};

		assert_eq!(
			vec![(
				SlotKey::Hand(Side::Off),
				SkillComboTree {
					skill: Skill {
						name: "alternate skill",
						..default()
					},
					next: SkillComboNext::Alternate {
						slot_key: SlotKey::Hand(Side::Main),
						skill: skill.clone().map_data(|_| ()),
					},
				},
			)],
			next.to_vec(&skill)
		);
	}
}
