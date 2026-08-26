use bevy::prelude::*;
use macros::{SavableComponent, serde_model};

#[serde_model]
#[derive(Component, SavableComponent, Debug, PartialEq, Default, Clone)]
#[savable_component(id = "per frame fall speed")]
pub(crate) struct CharacterGravity(pub(crate) f32);
