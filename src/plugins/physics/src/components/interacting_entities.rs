use bevy::prelude::{Component, Entity};
use std::collections::HashSet;

#[derive(Component, Default, Debug, PartialEq, Clone)]
pub struct InteractingEntities(pub(crate) HashSet<Entity>);
