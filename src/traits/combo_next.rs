pub mod skill_combo_next;

use crate::{
	components::SlotKey,
	skill::{Active, Skill, SkillComboTree},
};

pub trait ComboNext<TAnimationKey>
where
	Self: Sized,
{
	fn to_vec(&self, skill: &Skill<TAnimationKey, Active>) -> Vec<(SlotKey, SkillComboTree<Self>)>;
}
