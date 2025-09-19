use crate::traits::accessors::get::Property;

#[derive(Debug, PartialEq, Eq, Hash, Default, Clone, Copy)]
pub enum SkillExecution {
	#[default]
	None,
	Active,
	Queued,
}

impl Property for SkillExecution {
	type TValue<'a> = Self;
}
