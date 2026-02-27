use bevy::prelude::*;
use common::components::persistent_entity::PersistentEntity;

#[derive(Component, Debug, PartialEq)]
pub(crate) struct MapObjectOfPersistent(pub(crate) PersistentEntity);
