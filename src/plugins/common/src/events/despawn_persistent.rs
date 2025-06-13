use crate::components::persistent_entity::PersistentEntity;
use bevy::prelude::*;

#[derive(Event, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) struct DespawnPersistent(pub(crate) PersistentEntity);
