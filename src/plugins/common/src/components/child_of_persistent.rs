use crate::components::persistent_entity::PersistentEntity;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Can be used to make child relationships persistent across game sessions
///
/// Inserts [`ChildOf`] on its [`Entity`] via the [`CommonPlugin`](crate::CommonPlugin)
#[derive(Component, Debug, PartialEq, Serialize, Deserialize, Clone)]
#[component(immutable)]
pub struct ChildOfPersistent(pub PersistentEntity);
