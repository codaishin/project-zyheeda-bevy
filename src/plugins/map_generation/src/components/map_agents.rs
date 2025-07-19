use bevy::{ecs::entity::EntityHashSet, prelude::*};
use common::traits::accessors::get::Getter;

#[derive(Component, Debug, PartialEq)]
#[relationship_target(relationship = MapAgentOf)]
pub struct MapAgents(EntityHashSet);

#[derive(Component, Debug, PartialEq)]
#[relationship(relationship_target = MapAgents)]
pub struct MapAgentOf(pub(crate) Entity);

impl Getter<Entity> for MapAgentOf {
	fn get(&self) -> Entity {
		self.0
	}
}
