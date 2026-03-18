use crate::{
	components::grid::Grid,
	mesh_grid_graph::{Clearance, NodeId},
};
use bevy::{
	color::palettes::css::{BLUE, GREEN, RED},
	prelude::*,
};
use std::ops::Deref;

pub(crate) fn draw(app: &mut App) {
	app.add_systems(Update, draw_vertices);
}

fn draw_vertices(grids: Query<&Grid>, mut gizmos: Gizmos) {
	for grid in grids {
		let graph = grid.deref();

		for (node, vert) in graph.vertices.iter().enumerate() {
			gizmos.sphere(
				Vec3::from(vert),
				0.1,
				match graph.clearance[node] {
					Clearance::NONE => RED,
					_ => GREEN,
				},
			);
		}

		for (node, neighbors) in graph.neighbors.iter().enumerate() {
			for NodeId(neighbor) in neighbors {
				gizmos.arrow(
					grid.vertices[node].into(),
					grid.vertices[*neighbor].into(),
					BLUE,
				);
			}
		}
	}
}
