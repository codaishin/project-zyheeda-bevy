use crate::traits::{handles_localization::Token, inspect_able::InspectMarker};

#[derive(Debug, PartialEq)]
pub struct SkillToken;

impl InspectMarker for SkillToken {
	type TFieldRef<'a> = &'a Token;
}
