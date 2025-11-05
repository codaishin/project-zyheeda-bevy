use bevy::prelude::*;
use common::traits::{accessors::get::GetProperty, handles_map_generation::AgentType};

#[derive(Component, Debug, PartialEq)]
pub struct WorldAgent(pub(crate) AgentType);

impl GetProperty<AgentType> for WorldAgent {
	fn get_property(&self) -> AgentType {
		self.0
	}
}
