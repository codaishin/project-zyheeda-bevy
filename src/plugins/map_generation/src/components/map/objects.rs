use bevy::{ecs::entity::EntityHashSet, prelude::*};
use common::components::persistent_entity::PersistentEntity;

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct MapObject;

#[derive(Component, Debug, PartialEq, Default)]
#[relationship_target(relationship = MapObjectOf)]
pub(crate) struct MapObjects(EntityHashSet);

#[derive(Component, Debug, PartialEq)]
#[relationship(relationship_target = MapObjects)]
pub(crate) struct MapObjectOf(pub(crate) Entity);

#[derive(Component, Debug, PartialEq)]
pub(crate) struct MapObjectOfPersistent(pub(crate) PersistentEntity);
