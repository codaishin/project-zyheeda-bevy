use crate::components::{map::objects::MapObject, spawner_active::SpawnerActive};
use bevy::prelude::*;
use common::prelude::*;

#[derive(Component, Debug, PartialEq)]
#[require(SpawnerActive, MapObject)]
pub(crate) struct Spawner<T>(pub(crate) T)
where
	T: PrefabType;
