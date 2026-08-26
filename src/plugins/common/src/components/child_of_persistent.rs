use crate::components::persistent_entity::PersistentEntity;
use bevy::prelude::*;
use macros::{SavableComponent, serde_model};

/// Can be used to make child relationships persistent across game sessions
///
/// Inserts [`ChildOf`] on its [`Entity`] via the [`CommonPlugin`](crate::CommonPlugin)
#[serde_model]
#[derive(Component, SavableComponent, Debug, PartialEq, Clone)]
#[component(immutable)]
#[savable_component(id = "child of persistent")]
pub struct ChildOfPersistent(pub PersistentEntity);
