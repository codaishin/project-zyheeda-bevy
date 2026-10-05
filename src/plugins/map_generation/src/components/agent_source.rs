use bevy::prelude::*;
use common::prelude::*;

#[derive(Component, Debug, PartialEq)]
pub(crate) struct AgentSource(pub(crate) AgentType);
