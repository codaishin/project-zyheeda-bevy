use bevy::{ecs::entity::EntityHashSet, prelude::*};
use common::{
	components::persistent_entity::PersistentEntity,
	traits::accessors::get::GetProperty,
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, PartialEq)]
#[relationship_target(relationship = GridAgentOf)]
pub struct GridAgents(EntityHashSet);

#[derive(Component, Debug, PartialEq)]
#[relationship(relationship_target = GridAgents)]
pub struct GridAgentOf(pub(crate) Entity);

impl GetProperty<Entity> for GridAgentOf {
	fn get_property(&self) -> Entity {
		self.0
	}
}

#[derive(Component, SavableComponent, Debug, PartialEq, Serialize, Deserialize, Clone)]
#[component(immutable)]
pub struct AgentOfPersistentMap(pub(crate) PersistentEntity);
