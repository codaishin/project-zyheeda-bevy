pub mod skill_combo_next;

use crate::{
	components::SlotKey,
	skill::{Active, Skill, SkillComboTree},
};

pub trait ComboNext
where
	Self: Sized,
{
	fn to_branches(&self, skill: &Skill<Active>) -> Vec<(SlotKey, SkillComboTree<Self>)>;
}
