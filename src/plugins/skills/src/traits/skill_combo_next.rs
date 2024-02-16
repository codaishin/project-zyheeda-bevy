use super::ComboNext;
use crate::{
	components::{SideUnset, SlotKey},
	skill::{Active, PlayerSkills, Skill, SkillComboNext, SkillComboTree},
};

impl ComboNext<PlayerSkills<SideUnset>> for SkillComboNext {
	fn to_vec(
		&self,
		skill: &Skill<PlayerSkills<SideUnset>, Active>,
	) -> Vec<(SlotKey, SkillComboTree<Self>)> {
		match &self {
			SkillComboNext::Tree(tree) => tree.clone().into_iter().collect(),
			SkillComboNext::Alternate {
				slot_key,
				skill: alternate_skill,
			} => {
				vec![(
					*slot_key,
					SkillComboTree {
						skill: alternate_skill.clone(),
						next: SkillComboNext::Alternate {
							slot_key: skill.data.slot_key,
							skill: skill.clone().map_data(|_| ()),
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
			SlotKey::Legs,
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
			data: Active {
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
