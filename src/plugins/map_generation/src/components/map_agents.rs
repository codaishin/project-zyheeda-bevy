use bevy::{ecs::entity::EntityHashSet, prelude::*};
use common::components::persistent_entity::PersistentEntity;
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, PartialEq)]
#[relationship_target(relationship = GridAgentOf)]
pub struct GridAgents(EntityHashSet);

#[derive(Component, Debug, PartialEq)]
#[relationship(relationship_target = GridAgents)]
pub struct GridAgentOf(pub(crate) Entity);

impl From<&GridAgentOf> for Entity {
	fn from(GridAgentOf(entity): &GridAgentOf) -> Self {
		*entity
	}
}

#[derive(Component, SavableComponent, Debug, PartialEq, Serialize, Deserialize, Clone)]
#[component(immutable)]
pub struct AgentOfPersistentMap(pub(crate) PersistentEntity);
