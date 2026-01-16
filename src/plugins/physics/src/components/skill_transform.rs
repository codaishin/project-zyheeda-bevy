use bevy::{ecs::entity::EntityHashSet, prelude::*};

#[derive(Component, Debug, PartialEq, Default)]
#[relationship_target(relationship = SkillTransformOf)]
pub(crate) struct SkillTransforms(EntityHashSet);

#[derive(Component, Debug, PartialEq)]
#[relationship(relationship_target = SkillTransforms)]
#[require(Transform)]
pub(crate) struct SkillTransformOf(pub(crate) Entity);
