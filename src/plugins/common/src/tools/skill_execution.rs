use crate::traits::inspect_able::InspectMarker;

#[derive(Debug, PartialEq, Eq, Hash, Default, Clone, Copy)]
pub enum SkillExecution {
	#[default]
	None,
	Active,
	Queued,
}

impl InspectMarker for SkillExecution {
	type TFieldRef<'a> = &'a SkillExecution;
}
