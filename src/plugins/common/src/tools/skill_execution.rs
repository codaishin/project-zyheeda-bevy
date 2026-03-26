use crate::traits::accessors::get::ViewField;

#[derive(Debug, PartialEq, Eq, Hash, Default, Clone, Copy)]
pub enum SkillExecution {
	#[default]
	None,
	Active,
	Queued,
}

impl ViewField for SkillExecution {
	type TValue<'a> = Self;
}
