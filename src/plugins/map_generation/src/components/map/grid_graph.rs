use crate::grid_graph::GridGraph;
use bevy::prelude::*;
use std::marker::PhantomData;

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
