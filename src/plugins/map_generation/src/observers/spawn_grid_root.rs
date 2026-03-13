use crate::{
	components::{
		cells_ref::CellsRef,
		map::{grid_graph::MapGridGraph, objects::MapObjectOf},
	},
	square_grid_graph::SquareGridGraph,
};
use bevy::prelude::*;
use common::{
	traits::{accessors::get::TryApplyOn, thread_safe::ThreadSafe},
	zyheeda_commands::ZyheedaCommands,
};

impl<TCell> MapGridGraph<TCell>
where
	TCell: ThreadSafe,
{
	pub(crate) fn spawn_child<TGrid>(
		on_insert: On<Insert, Self>,
		maps: Query<(&Self, Option<&Children>)>,
		grids: Query<&TGrid>,
		mut commands: ZyheedaCommands,
	) where
		for<'a> TGrid: Component + From<&'a SquareGridGraph>,
	{
		let target = on_insert.entity;
		let Ok((map, children)) = maps.get(target) else {
			return;
		};

		for grid in old(children, grids) {
			commands.try_apply_on(&grid, |e| e.try_despawn());
		}

		commands.spawn((
			ChildOf(target),
			MapObjectOf(target),
			TGrid::from(map.graph()),
			CellsRef::<TCell>::from_grid_definition(target),
		));
	}
}

fn old<TGrid>(children: Option<&Children>, grids: Query<&TGrid>) -> Vec<Entity>
where
	TGrid: Component,
{
	match children {
		Some(children) => children.iter().filter(|c| grids.contains(*c)).collect(),
		None => vec![],
	}
}

#[cfg(test)]
mod tests {
	use crate::square_grid_graph::{
		Obstacles,
		context::{CellCount, CellDistance, SquareGridContext},
	};

	use super::*;
	use common::traits::handles_map_generation::GroundPosition;
	use macros::new_valid;
	use std::collections::HashMap;
	use testing::{SingleThreadedApp, assert_children_count};

	#[derive(TypePath, Debug, PartialEq)]
	struct _Cell;

	#[derive(Component, Debug, PartialEq)]
	struct _Grid(SquareGridGraph);

	impl From<&SquareGridGraph> for _Grid {
		fn from(graph: &SquareGridGraph) -> Self {
			Self(graph.clone())
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(MapGridGraph::<_Cell>::spawn_child::<_Grid>);

		app
	}

	#[test]
	fn spawn_grid() {
		let graph = SquareGridGraph {
			nodes: HashMap::from([((0, 0), GroundPosition::ZERO)]),
			extra: Obstacles::default(),
			context: SquareGridContext {
				cell_count_x: new_valid!(CellCount, 1),
				cell_count_z: new_valid!(CellCount, 1),
				cell_distance: new_valid!(CellDistance, 2.),
			},
		};
		let mut app = setup();

		let entity = app
			.world_mut()
			.spawn(MapGridGraph::<_Cell>::from(graph.clone()))
			.id();

		let [grid] = assert_children_count!(1, app, entity);
		assert_eq!(Some(&_Grid(graph)), grid.get::<_Grid>());
	}

	#[test]
	fn spawn_grid_cell_type() {
		let graph = SquareGridGraph {
			nodes: HashMap::from([((0, 0), GroundPosition::ZERO)]),
			extra: Obstacles::default(),
			context: SquareGridContext {
				cell_count_x: new_valid!(CellCount, 1),
				cell_count_z: new_valid!(CellCount, 1),
				cell_distance: new_valid!(CellDistance, 2.),
			},
		};
		let mut app = setup();

		let entity = app
			.world_mut()
			.spawn(MapGridGraph::<_Cell>::from(graph.clone()))
			.id();

		let [grid] = assert_children_count!(1, app, entity);
		assert_eq!(
			Some(&CellsRef::<_Cell>::from_grid_definition(entity)),
			grid.get::<CellsRef<_Cell>>()
		);
	}

	#[test]
	fn replace_old_root_when_inserting_again() {
		#[derive(Component, Debug, PartialEq)]
		struct _Child;

		let graph_a = SquareGridGraph {
			nodes: HashMap::from([((0, 0), GroundPosition::ZERO)]),
			extra: Obstacles::default(),
			context: SquareGridContext {
				cell_count_x: new_valid!(CellCount, 1),
				cell_count_z: new_valid!(CellCount, 1),
				cell_distance: new_valid!(CellDistance, 2.),
			},
		};
		let graph_b = SquareGridGraph {
			nodes: HashMap::from([((0, 0), GroundPosition::ZERO)]),
			extra: Obstacles::default(),
			context: SquareGridContext {
				cell_count_x: new_valid!(CellCount, 1),
				cell_count_z: new_valid!(CellCount, 1),
				cell_distance: new_valid!(CellDistance, 10.),
			},
		};
		let mut app = setup();

		let entity = app
			.world_mut()
			.spawn_empty()
			.with_child(_Child)
			.insert(MapGridGraph::<_Cell>::from(graph_a))
			.insert(MapGridGraph::<_Cell>::from(graph_b.clone()))
			.id();

		let [grid] = assert_children_count!(1, app, entity, |entity| entity.get::<_Grid>());
		assert_eq!(&_Grid(graph_b), grid);
	}

	#[test]
	fn spawn_map_child() {
		let graph = SquareGridGraph {
			nodes: HashMap::from([((0, 0), GroundPosition::ZERO)]),
			extra: Obstacles::default(),
			context: SquareGridContext {
				cell_count_x: new_valid!(CellCount, 1),
				cell_count_z: new_valid!(CellCount, 1),
				cell_distance: new_valid!(CellDistance, 2.),
			},
		};
		let mut app = setup();

		let entity = app
			.world_mut()
			.spawn(MapGridGraph::<_Cell>::from(graph.clone()))
			.id();

		let [grid] = assert_children_count!(1, app, entity);
		assert_eq!(Some(&MapObjectOf(entity)), grid.get::<MapObjectOf>());
	}
}
