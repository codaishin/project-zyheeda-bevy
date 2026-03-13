use bevy::{ecs::entity::EntityHashSet, prelude::*};
use common::traits::accessors::get::GetProperty;
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(SavableComponent, Component, Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
#[savable_component(id = "grid agent")]
pub struct GridAgent;

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
