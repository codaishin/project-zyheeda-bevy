use bevy::{ecs::entity::EntityHashSet, prelude::*};
use bevy_rapier3d::prelude::*;
use common::components::persistent_entity::PersistentEntity;

/// Marks an entity as the target for interactions like damaging effects, healing, etc.
#[derive(Component, PartialEq, Debug, Default)]
#[component(immutable)]
#[require(PersistentEntity)]
pub(crate) struct InteractionTarget;

/// Links a [`Collider`] entity to the corresponding [`InteractionTarget`] entity.
#[derive(Component, PartialEq, Debug)]
#[relationship(relationship_target = InteractionColliders)]
#[require(Collider, Transform, ActiveEvents, ActiveCollisionTypes)]
pub(crate) struct ColliderOfInteractionTarget(pub(crate) Entity);

#[derive(Component, PartialEq, Debug)]
#[relationship_target(relationship = ColliderOfInteractionTarget)]
pub(crate) struct InteractionColliders(EntityHashSet);
