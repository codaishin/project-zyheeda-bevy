use bevy::prelude::*;
use common::traits::handles_agents::{AgentActionTarget, CurrentAction};
use std::collections::HashMap;

#[derive(Component, Debug, PartialEq)]
pub struct Actions(pub(crate) HashMap<CurrentAction, AgentActionTarget>);
