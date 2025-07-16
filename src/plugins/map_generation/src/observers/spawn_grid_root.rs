use crate::{
	components::{cells_ref::CellsRef, map::grid_graph::MapGridGraph},
	grid_graph::GridGraph,
};
use bevy::prelude::*;
use common::traits::{thread_safe::ThreadSafe, try_despawn::TryDespawn};

impl<TCell> MapGridGraph<TCell>
where
	TCell: ThreadSafe,
{
	pub(crate) fn spawn_child<TGrid>(
		trigger: Trigger<OnInsert, Self>,
		maps: Query<(&Self, Option<&Children>)>,
		grids: Query<&TGrid>,
		mut commands: Commands,
	) where
		for<'a> TGrid: Component + From<&'a GridGraph>,
	{
		let target = trigger.target();
		let Ok((map, children)) = maps.get(target) else {
			return;
		};

		for grid in old(children, grids) {
			commands.try_despawn(grid);
		}

		commands.spawn((
			ChildOf(target),
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
	use super::*;
	use crate::grid_graph::{
		Obstacles,
		grid_context::{GridContext, GridDefinition},
	};
	use std::collections::HashMap;
	use testing::{SingleThreadedApp, assert_count, get_children};

	#[derive(TypePath, Debug, PartialEq)]
	struct _Cell;

	#[derive(Component, Debug, PartialEq)]
	struct _Grid(GridGraph);

	impl From<&GridGraph> for _Grid {
		fn from(graph: &GridGraph) -> Self {
			Self(graph.clone())
		}
	}

	fn contains<T>(entity: &EntityRef) -> bool
	where
		T: Component,
	{
		entity.contains::<T>()
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(MapGridGraph::<_Cell>::spawn_child::<_Grid>);

		app
	}

	#[test]
	fn spawn_grid() {
		let graph = GridGraph {
			nodes: HashMap::from([((0, 0), Vec3::ZERO)]),
			extra: Obstacles::default(),
			context: GridContext::try_from(GridDefinition {
				cell_count_x: 1,
				cell_count_z: 1,
				cell_distance: 2.,
			})
			.expect("INVALID GRID DEFINITION"),
		};
		let mut app = setup();

		let entity = app
			.world_mut()
			.spawn(MapGridGraph::<_Cell>::from(graph.clone()))
			.id();

		let [grid] = assert_count!(1, get_children!(app, entity));
		assert_eq!(Some(&_Grid(graph)), grid.get::<_Grid>());
	}

	#[test]
	fn spawn_grid_cell_type() {
		let graph = GridGraph {
			nodes: HashMap::from([((0, 0), Vec3::ZERO)]),
			extra: Obstacles::default(),
			context: GridContext::try_from(GridDefinition {
				cell_count_x: 1,
				cell_count_z: 1,
				cell_distance: 2.,
			})
			.expect("INVALID GRID DEFINITION"),
		};
		let mut app = setup();

		let entity = app
			.world_mut()
			.spawn(MapGridGraph::<_Cell>::from(graph.clone()))
			.id();

		let [grid] = assert_count!(1, get_children!(app, entity));
		assert_eq!(
			Some(&CellsRef::<_Cell>::from_grid_definition(entity)),
			grid.get::<CellsRef<_Cell>>()
		);
	}

	#[test]
	fn replace_old_root_when_inserting_again() {
		#[derive(Component, Debug, PartialEq)]
		struct _Child;

		let graph_a = GridGraph {
			nodes: HashMap::from([((0, 0), Vec3::ZERO)]),
			extra: Obstacles::default(),
			context: GridContext::try_from(GridDefinition {
				cell_count_x: 1,
				cell_count_z: 1,
				cell_distance: 2.,
			})
			.expect("INVALID GRID DEFINITION"),
		};
		let graph_b = GridGraph {
			nodes: HashMap::from([((0, 0), Vec3::ZERO)]),
			extra: Obstacles::default(),
			context: GridContext::try_from(GridDefinition {
				cell_count_x: 1,
				cell_count_z: 1,
				cell_distance: 10.,
			})
			.expect("INVALID GRID DEFINITION"),
		};
		let mut app = setup();

		let entity = app
			.world_mut()
			.spawn_empty()
			.with_child(_Child)
			.insert(MapGridGraph::<_Cell>::from(graph_a))
			.insert(MapGridGraph::<_Cell>::from(graph_b.clone()))
			.id();

		assert_count!(1, get_children!(app, entity).filter(contains::<_Child>));
		let [grid] = assert_count!(1, get_children!(app, entity).filter(contains::<_Grid>));
		assert_eq!(Some(&_Grid(graph_b)), grid.get::<_Grid>());
	}
}
