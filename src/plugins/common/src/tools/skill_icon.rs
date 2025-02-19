use bevy::prelude::*;

use crate::traits::inspect_able::InspectMarker;

#[derive(Debug, PartialEq)]
pub struct SkillIcon;

impl InspectMarker for SkillIcon {
	type TFieldRef<'a> = &'a Option<Handle<Image>>;
}
