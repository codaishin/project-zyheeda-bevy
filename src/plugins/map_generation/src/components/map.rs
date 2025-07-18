pub(crate) mod agents;
pub(crate) mod cells;
pub(crate) mod demo_map;
pub(crate) mod folder;
pub(crate) mod grid_graph;
pub(crate) mod image;

use bevy::prelude::*;

#[derive(Component, Debug, PartialEq, Clone, Default)]
pub(crate) struct Map;
