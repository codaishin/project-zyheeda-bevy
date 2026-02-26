pub(crate) mod agents;
pub(crate) mod bay;
pub(crate) mod cells;
pub(crate) mod demo_map;
pub(crate) mod folder;
pub(crate) mod grid_graph;
pub(crate) mod image;
pub(crate) mod objects;

use bevy::prelude::*;
use common::{
	components::persistent_entity::PersistentEntity,
	errors::Unreachable,
	traits::handles_custom_assets::TryLoadFrom,
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Default)]
#[require(PersistentEntity)]
#[savable_component(id = "map", dto = MapDto)]
pub(crate) struct Map {
	pub(crate) created_from_save: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct MapDto;

impl From<Map> for MapDto {
	fn from(_: Map) -> Self {
		Self
	}
}

impl TryLoadFrom<MapDto> for Map {
	type TInstantiationError = Unreachable;

	fn try_load_from<TLoadAsset>(
		_: MapDto,
		_: &mut TLoadAsset,
	) -> Result<Self, Self::TInstantiationError> {
		Ok(Self {
			created_from_save: true,
		})
	}
}
