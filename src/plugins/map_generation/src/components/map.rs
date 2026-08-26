pub(crate) mod agents;
pub(crate) mod level;
pub(crate) mod objects;

use crate::components::map::objects::MapObjects;
use bevy::prelude::*;
use common::prelude::*;
use macros::{SavableComponent, serde_model};
use std::{borrow::Borrow, collections::HashSet};

#[serde_model]
#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Default)]
#[require(PersistentEntity, MapObjects)]
#[savable_component(id = "map")]
pub(crate) struct Map {
	pub(crate) disabled_object_sources: HashSet<MapObjectSource>,
}

#[serde_model]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Default)]
pub(crate) struct MapObjectSource(pub(crate) String);

impl Borrow<String> for MapObjectSource {
	fn borrow(&self) -> &String {
		&self.0
	}
}
