use bevy::{ecs::entity::EntityHashSet, prelude::*};

#[derive(Component, Default)]
#[relationship_target(relationship = ChildMeshOf)]
pub(crate) struct ChildMeshes(EntityHashSet);

#[derive(Component)]
#[relationship(relationship_target = ChildMeshes)]
pub(crate) struct ChildMeshOf(pub(crate) Entity);
