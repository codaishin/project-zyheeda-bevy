use bevy::prelude::*;
use common::traits::{accessors::get::GetProperty, handles_map_generation::AgentType};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[component(immutable)]
pub(crate) struct AgentTag(pub(crate) AgentType);

impl GetProperty<AgentType> for AgentTag {
	fn get_property(&self) -> AgentType {
		self.0
	}
}
