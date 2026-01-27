use bevy::{ecs::entity::EntityHashSet, prelude::*};

#[derive(Component, Debug, PartialEq)]
#[relationship_target(relationship = LifetimeTiedTo)]
pub(crate) struct TiedLifetimes(EntityHashSet);

#[derive(Component, Debug, PartialEq)]
#[relationship(relationship_target = TiedLifetimes)]
pub(crate) struct LifetimeTiedTo(pub(crate) Entity);
