use crate::{components::map::objects::MapObject, mesh_grid_graph::MeshGridGraph};
use bevy::prelude::*;
use std::ops::Deref;

#[derive(Component, Debug, PartialEq)]
#[require(Name = Self::name(), Transform, Visibility, MapObject)]
#[component(immutable)]
pub struct Grid<const SUBDIVISIONS: u8 = 0, TGraph = MeshGridGraph> {
	graph: TGraph,
}

impl<const SUBDIVISIONS: u8, TGraph> Grid<SUBDIVISIONS, TGraph> {
	fn name() -> String {
		format!("Grid (subdivisions: {SUBDIVISIONS})")
	}
}

impl Default for Grid {
	fn default() -> Self {
		Self {
			graph: Default::default(),
		}
	}
}

impl<TGraph> From<&TGraph> for Grid<0, TGraph>
where
	TGraph: Clone,
{
	fn from(graph: &TGraph) -> Self {
		Grid {
			graph: graph.clone(),
		}
	}
}

impl<TGraph> From<TGraph> for Grid<0, TGraph> {
	fn from(graph: TGraph) -> Self {
		Grid { graph }
	}
}

impl From<&Grid<0, MeshGridGraph>> for MeshGridGraph {
	fn from(value: &Grid<0, MeshGridGraph>) -> Self {
		value.graph.clone()
	}
}

impl<const SUBDIVISIONS: u8, TGraph> Deref for Grid<SUBDIVISIONS, TGraph> {
	type Target = TGraph;

	fn deref(&self) -> &Self::Target {
		&self.graph
	}
}
