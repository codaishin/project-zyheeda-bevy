use bevy::prelude::*;
use common::components::persistent_entity::PersistentEntity;

#[derive(Component, Debug, PartialEq)]
#[require(PersistentEntity)]
pub struct Skill;
