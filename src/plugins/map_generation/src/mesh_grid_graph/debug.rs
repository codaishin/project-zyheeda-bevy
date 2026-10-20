use crate::{
	components::{grid::Grid, nav_mesh_debug_agent::NavMeshDebugAgent},
	mesh_grid_graph::{Clearance, NodeId},
};
use bevy::{
	color::palettes::css::{BLUE, GREEN, RED},
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::prelude::*;
use std::ops::Deref;

pub(crate) fn draw<TInput>(app: &mut App)
where
	TInput: ThreadSafe + HandlesInput + SystemSetDefinition,
{
	app.init_resource::<NavMeshGizmos>().add_systems(
		Update,
		(set_nav_mesh_gizmos::<TInput::TInput>, draw_vertices)
			.chain()
			.after_plugin(TInput::SYSTEMS),
	);
}

fn set_nav_mesh_gizmos<TInput>(
	input: Option<StaticSystemParam<TInput>>,
	mut gizmos: ResMut<NavMeshGizmos>,
) where
	TInput: for<'w, 's> SystemParam<Item<'w, 's>: GetInputState>,
{
	let Some(input) = input else {
		return;
	};

	let state = input.get_input_state(DebugKey::RotateNavMeshDebug);

	if state != InputState::just_pressed() {
		return;
	}

	gizmos.rotate();
}

fn draw_vertices(
	mut gizmos: Gizmos,
	gizmos_setting: Res<NavMeshGizmos>,
	grids: Query<&Grid>,
	agents: Query<&GlobalTransform, With<NavMeshDebugAgent>>,
) {
	for grid in grids {
		let graph = grid.deref();

		for (node, neighbors) in graph.neighbors.iter().enumerate() {
			let node_position = match *gizmos_setting {
				NavMeshGizmos::None => None,
				NavMeshGizmos::AroundAgents => {
					let node_position = Vec3::from(grid.vertices[node]);
					let in_range_of_agent = agents.iter().any(|agent_transform| {
						(node_position - agent_transform.translation()).length() < 10.
					});

					match in_range_of_agent {
						true => Some(node_position),
						false => None,
					}
				}
				NavMeshGizmos::All => Some(Vec3::from(grid.vertices[node])),
			};

			let Some(node_position) = node_position else {
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

#[derive(Resource, Default)]
enum NavMeshGizmos {
	#[default]
	None,
	AroundAgents,
	All,
}

impl NavMeshGizmos {
	fn rotate(&mut self) {
		*self = match self {
			Self::None => Self::AroundAgents,
			Self::AroundAgents => Self::All,
			Self::All => Self::None,
		}
	}
}
