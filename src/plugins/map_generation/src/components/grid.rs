use crate::{components::map::objects::MapObject, mesh_grid_graph::MeshGridGraph};
use bevy::prelude::*;
use std::ops::Deref;

#[derive(Component, Debug, PartialEq)]
#[require(Name = "Grid", Transform, Visibility, MapObject)]
#[component(immutable)]
pub struct Grid<TGraph = MeshGridGraph> {
	graph: TGraph,
}

impl Default for Grid {
	fn default() -> Self {
		Self {
			graph: Default::default(),
		}
	}
}

impl<TGraph> From<&TGraph> for Grid<TGraph>
where
	TGraph: Clone,
{
	fn from(graph: &TGraph) -> Self {
		Grid {
			graph: graph.clone(),
		}
	}
}

impl<TGraph> From<TGraph> for Grid<TGraph> {
	fn from(graph: TGraph) -> Self {
		Grid { graph }
	}
}

impl From<&Grid> for MeshGridGraph {
	fn from(value: &Grid) -> Self {
		value.graph.clone()
	}
}

impl<TGraph> Deref for Grid<TGraph> {
	type Target = TGraph;

	fn deref(&self) -> &Self::Target {
		&self.graph
	}
}
