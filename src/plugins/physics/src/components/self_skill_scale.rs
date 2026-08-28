use bevy::prelude::*;
use common::prelude::*;

#[derive(Component, Debug, PartialEq)]
pub(crate) struct SelfSkillScale(pub(crate) Scale<3>);
