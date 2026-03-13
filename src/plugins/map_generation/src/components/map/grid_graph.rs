use crate::square_grid_graph::SquareGridGraph;
use bevy::prelude::*;
use std::marker::PhantomData;

#[derive(Component, Debug, PartialEq)]
#[component(immutable)]
#[require(Transform, Visibility)]
pub(crate) struct MapGridGraph<TCell> {
	graph: SquareGridGraph,
	_p: PhantomData<TCell>,
}

impl<TCell> From<SquareGridGraph> for MapGridGraph<TCell> {
	fn from(graph: SquareGridGraph) -> Self {
		Self {
			graph,
			_p: PhantomData,
		}
	}
}

impl<TCell> MapGridGraph<TCell> {
	pub(crate) fn graph(&self) -> &SquareGridGraph {
		&self.graph
	}
}
