pub(crate) mod agents;
pub(crate) mod level;
pub(crate) mod objects;

use crate::components::map::objects::MapObjects;
use bevy::prelude::*;
use common::components::persistent_entity::PersistentEntity;
use macros::SavableComponent;
use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, collections::HashSet};

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
#[require(PersistentEntity, MapObjects)]
#[savable_component(id = "map")]
pub(crate) struct Map {
	pub(crate) disabled_object_sources: HashSet<MapObjectSource>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Default, Serialize, Deserialize)]
pub(crate) struct MapObjectSource(pub(crate) String);

impl Borrow<String> for MapObjectSource {
	fn borrow(&self) -> &String {
		&self.0
	}
}
