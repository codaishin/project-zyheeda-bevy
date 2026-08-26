use bevy::{ecs::entity::EntityHashSet, prelude::*};
use common::prelude::*;
use macros::{SavableComponent, serde_model};

#[serde_model]
#[derive(SavableComponent, Component, Debug, PartialEq, Default, Clone)]
#[savable_component(id = "grid agent")]
pub struct GridAgent;

#[derive(Component, Debug, PartialEq)]
#[relationship_target(relationship = GridAgentOf)]
pub struct GridAgents(EntityHashSet);

#[derive(Component, Debug, PartialEq)]
#[relationship(relationship_target = GridAgents)]
pub struct GridAgentOf(pub(crate) Entity);

impl View<Entity> for GridAgentOf {
	fn view(&self) -> Entity {
		self.0
	}
}
