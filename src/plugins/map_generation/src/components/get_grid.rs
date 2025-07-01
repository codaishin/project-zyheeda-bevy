use bevy::prelude::*;
use common::traits::{accessors::get::Getter, handles_map_generation::Map};

#[derive(Component, Debug, Default, PartialEq, Eq, Hash, Clone, Copy)]
pub struct GetGrid(pub(crate) Option<Entity>);

impl Getter<Map> for GetGrid {
	fn get(&self) -> Map {
		match self.0 {
			Some(map) => Map::Entity(map),
			None => Map::None,
		}
	}
}
