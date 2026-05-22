use crate::components::{map::objects::MapObject, spawner_active::SpawnerActive};
use bevy::prelude::*;
use common::traits::handles_map_generation::PrefabType;

#[derive(Component, Debug, PartialEq)]
#[require(SpawnerActive, MapObject)]
pub(crate) struct Spawner<T>(pub(crate) T)
where
	T: PrefabType;
