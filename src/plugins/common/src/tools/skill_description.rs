use crate::traits::inspect_able::InspectMarker;

#[derive(Debug, PartialEq)]
pub struct SkillDescription;

impl InspectMarker for SkillDescription {
	type TFieldRef<'a> = String;
}
