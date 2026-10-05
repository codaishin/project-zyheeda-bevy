use crate::{
	components::{grid::Grid, nav_mesh_debug_agent::NavMeshDebugAgent},
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

fn draw_vertices(
	mut gizmos: Gizmos,
	grids: Query<&Grid>,
	agents: Query<&GlobalTransform, With<NavMeshDebugAgent>>,
) {
	for grid in grids {
		let graph = grid.deref();

		for (node, neighbors) in graph.neighbors.iter().enumerate() {
			let node_position = Vec3::from(grid.vertices[node]);
			let debug = agents.iter().any(|agent_transform| {
				(node_position - agent_transform.translation()).length() < 10.
			});

			if !debug {
				continue;
			};

			gizmos.sphere(
				node_position,
				0.1,
				match graph.clearance[node] {
					Clearance::NONE => RED,
					_ => GREEN,
				},
			);

			for NodeId(neighbor) in neighbors {
				let neighbor_position = Vec3::from(grid.vertices[*neighbor]);
				gizmos.arrow(node_position, neighbor_position, BLUE);
			}
		}
	}
}
