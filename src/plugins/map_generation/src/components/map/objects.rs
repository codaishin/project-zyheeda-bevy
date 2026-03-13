use bevy::{ecs::entity::EntityHashSet, prelude::*};
use common::components::persistent_entity::PersistentEntity;
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct MapObject;

#[derive(Component, Debug, PartialEq, Default)]
#[relationship_target(relationship = MapObjectOf)]
pub(crate) struct MapObjects(EntityHashSet);

#[derive(Component, Debug, PartialEq)]
#[relationship(relationship_target = MapObjects)]
pub(crate) struct MapObjectOf(pub(crate) Entity);

#[derive(SavableComponent, Component, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[savable_component(id = "map object of")]
pub(crate) struct PersistentMapObject {
	pub(crate) map: PersistentEntity,
}
