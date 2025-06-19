pub(crate) mod demo_map;

use crate::map_cells::MapCells;
use bevy::prelude::*;
use common::traits::{load_asset::Path, thread_safe::ThreadSafe};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

#[derive(Component, Debug, PartialEq, Serialize, Deserialize)]
#[component(immutable)]
pub(crate) struct MapAssetPath<TCell>
where
	TCell: TypePath + ThreadSafe,
{
	pub(crate) asset_path: Path,
	#[serde(skip)]
	_p: PhantomData<TCell>,
}

impl<TCell> From<Path> for MapAssetPath<TCell>
where
	TCell: TypePath + ThreadSafe,
{
	fn from(asset_path: Path) -> Self {
		Self {
			asset_path,
			_p: PhantomData,
		}
	}
}

#[derive(Component, Debug, PartialEq)]
#[component(immutable)]
pub(crate) struct MapAssetCells<TCell>
where
	TCell: TypePath + ThreadSafe,
{
	cells: Handle<MapCells<TCell>>,
}

impl<TCell> From<Handle<MapCells<TCell>>> for MapAssetCells<TCell>
where
	TCell: TypePath + ThreadSafe,
{
	fn from(cells: Handle<MapCells<TCell>>) -> Self {
		Self { cells }
	}
}
