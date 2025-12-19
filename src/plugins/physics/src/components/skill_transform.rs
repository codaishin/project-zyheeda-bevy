use bevy::{ecs::entity::EntityHashSet, prelude::*};

/// Mark an entity as a target for local transform updates.
///
/// This is used to control when local transforms of skills update instead of needing
/// to wait for bevy's transform propagation
///
/// <div class="warning">
///   Insert this only after the parent/child relationship in the skill hierarchy is
///   set, otherwise the related observer will not pick this insertion up
/// </div>
#[derive(Component, Debug, PartialEq)]
pub(crate) struct SkillTransform;

#[derive(Component, Debug, PartialEq)]
#[relationship_target(relationship = SkillTransformOf)]
pub(crate) struct SkillTransforms(EntityHashSet);

#[derive(Component, Debug, PartialEq)]
#[relationship(relationship_target = SkillTransforms)]
pub(crate) struct SkillTransformOf(pub(crate) Entity);
