use common::prelude::*;

pub(crate) trait UpdateComboSkills<TSkill> {
	fn update_combo_skills<'a, TComboIter>(&'a mut self, combos: TComboIter)
	where
		TSkill: 'a,
		TComboIter: Iterator<Item = (Vec<SlotKey>, Option<&'a TSkill>)>;
}
