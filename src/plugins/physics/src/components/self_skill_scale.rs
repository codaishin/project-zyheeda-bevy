use bevy::prelude::*;
use common::prelude::*;
use macros::{SavableComponent, serde_model};

#[serde_model]
#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Copy)]
#[savable_component(id = "self_skill_scale", has_priority)]
pub(crate) struct SelfSkillScale(pub(crate) Scale<3>);
