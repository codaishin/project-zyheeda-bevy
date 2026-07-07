pub(crate) mod agents;
pub(crate) mod level;
pub(crate) mod objects;

use crate::components::map::objects::MapObjects;
use bevy::prelude::*;
use common::{
	components::persistent_entity::PersistentEntity,
	traits::handles_map_generation::{AgentType, InteractiveType},
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
#[require(PersistentEntity, MapObjects)]
#[savable_component(id = "map")]
pub(crate) struct Map {
	pub(crate) persistent: HashSet<MapObjectType>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub(crate) enum MapObjectType {
	Agent(AgentType),
	InteractiveType(InteractiveType),
}

impl From<AgentType> for MapObjectType {
	fn from(agent: AgentType) -> Self {
		Self::Agent(agent)
	}
}

impl From<InteractiveType> for MapObjectType {
	fn from(interactive: InteractiveType) -> Self {
		Self::InteractiveType(interactive)
	}
}
