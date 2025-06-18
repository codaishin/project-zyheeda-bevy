pub(crate) mod demo_map;

use crate::{grid_graph::GridGraph, map_cells::MapCells};
use bevy::prelude::*;
use common::traits::{load_asset::Path, thread_safe::ThreadSafe};

#[derive(Component, Debug, PartialEq)]
pub(crate) struct Map<TCell>
where
	TCell: TypePath + ThreadSafe,
{
	asset_path: Path,
	cells: Option<Handle<MapCells<TCell>>>,
	graph: Option<GridGraph>,
}

impl<TCell> Map<TCell>
where
	TCell: TypePath + ThreadSafe,
{
	pub(crate) fn from_asset(asset_path: Path) -> Self {
		Self {
			asset_path,
			cells: None,
			graph: None,
		}
	}
}
