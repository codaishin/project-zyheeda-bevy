use bevy::{ecs::entity::EntityHashSet, prelude::*};
use common::{components::persistent_entity::PersistentEntity, traits::accessors::get::Getter};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, PartialEq)]
#[relationship_target(relationship = GridAgentOf)]
pub struct GridAgents(EntityHashSet);

#[derive(Component, Debug, PartialEq)]
#[relationship(relationship_target = GridAgents)]
pub struct GridAgentOf(Entity);

impl Getter<Entity> for GridAgentOf {
	fn get(&self) -> Entity {
		self.0
	}
}

#[derive(Component, SavableComponent, Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct AgentOfPersistentMap(pub(crate) PersistentEntity);
