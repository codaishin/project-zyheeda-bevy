pub(crate) mod demo_map;

use crate::{grid_graph::GridGraph, map_cells::MapCells};
use bevy::prelude::*;
use common::traits::{
	handles_load_tracking::Loaded,
	handles_saving::SavableComponent,
	load_asset::Path,
	thread_safe::ThreadSafe,
};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

#[derive(Component, Debug, PartialEq, Serialize, Deserialize)]
#[component(immutable)]
pub(crate) struct MapAssetPath<TCell> {
	pub(crate) asset_path: Path,
	#[serde(skip)]
	_p: PhantomData<TCell>,
}

impl<TCell> From<Path> for MapAssetPath<TCell> {
	fn from(asset_path: Path) -> Self {
		Self {
			asset_path,
			_p: PhantomData,
		}
	}
}

impl<TCell> Clone for MapAssetPath<TCell> {
	fn clone(&self) -> Self {
		Self {
			asset_path: self.asset_path.clone(),
			_p: PhantomData,
		}
	}
}

impl<TCell> SavableComponent for MapAssetPath<TCell>
where
	TCell: ThreadSafe,
{
	type TDto = Self;
}

#[derive(Component, Debug, PartialEq)]
#[component(immutable)]
pub(crate) struct MapAssetCells<TCell>
where
	TCell: TypePath + ThreadSafe,
{
	cells: Handle<MapCells<TCell>>,
}

impl<TCell> MapAssetCells<TCell>
where
	TCell: TypePath + ThreadSafe,
{
	pub(crate) fn cells(&self) -> &Handle<MapCells<TCell>> {
		&self.cells
	}
}

impl<TCell> MapAssetCells<TCell>
where
	TCell: TypePath + ThreadSafe,
{
	pub(crate) fn all_loaded(map_cells: Query<&Self>, asset_server: Res<AssetServer>) -> Loaded {
		Loaded(
			map_cells
				.iter()
				.all(|map_cells| asset_server.is_loaded_with_dependencies(map_cells.cells.id())),
		)
	}
}

impl<TCell> From<Handle<MapCells<TCell>>> for MapAssetCells<TCell>
where
	TCell: TypePath + ThreadSafe,
{
	fn from(cells: Handle<MapCells<TCell>>) -> Self {
		Self { cells }
	}
}

#[derive(Component, Debug, PartialEq)]
#[component(immutable)]
#[require(Transform, Visibility)]
pub(crate) struct MapGridGraph<TCell> {
	graph: GridGraph,
	_p: PhantomData<TCell>,
}

impl<TCell> From<GridGraph> for MapGridGraph<TCell> {
	fn from(graph: GridGraph) -> Self {
		Self {
			graph,
			_p: PhantomData,
		}
	}
}

impl<TCell> MapGridGraph<TCell> {
	pub(crate) fn graph(&self) -> &GridGraph {
		&self.graph
	}
}
