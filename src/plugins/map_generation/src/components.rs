pub(crate) mod agent_spawner;
pub(crate) mod floor_light;
pub(crate) mod grid;
pub(crate) mod map;
pub(crate) mod map_agents;
pub(crate) mod mesh_collider;
pub(crate) mod nav_grid;
pub(crate) mod nav_mesh;
pub(crate) mod wall_back;
pub(crate) mod wall_light;

use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct Unlit;
