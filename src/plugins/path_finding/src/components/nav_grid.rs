use crate::tools::nav_grid_node::NavGridNode;
use bevy::prelude::*;
use common::{
	tools::grid_cell_distance::GridCellDistance,
	traits::{
		handles_map_generation::NavCell,
		handles_path_finding::ComputePath,
		inspect_able::{InspectAble, InspectField},
		iterate::Iterate,
		thread_safe::ThreadSafe,
	},
};
use std::collections::HashSet;

#[derive(Component, Debug, PartialEq, Default)]
pub struct NavGrid<TMethod> {
	pub(crate) cells: Vec<NavCell>,
	pub(crate) cell_distance: f32,
	pub(crate) method: TMethod,
}

impl<TMethod> NavGrid<TMethod> {
	pub(crate) fn update_from<TMap>(mut maps: Query<(&TMap, &mut Self), Changed<TMap>>)
	where
		for<'a> TMap: Component + Iterate<TItem<'a> = &'a NavCell> + InspectAble<GridCellDistance>,
		TMethod: ThreadSafe + From<NavGridData>,
	{
		for (map, mut nav_grid) in &mut maps {
			let mut empty = true;
			let mut grid = NavGridData {
				min: NavGridNode::MAX,
				max: NavGridNode::MIN,
				obstacles: HashSet::default(),
			};
			let mut cells = vec![];

			for cell in map.iterate() {
				cells.push(*cell);
				empty = false;

				let node = NavGridNode::from(cell);
				if !cell.is_walkable {
					grid.obstacles.insert(node);
				}
				if node.x < grid.min.x {
					grid.min.x = node.x;
				}
				if node.x > grid.max.x {
					grid.max.x = node.x;
				}
				if node.y < grid.min.y {
					grid.min.y = node.y
				}
				if node.y > grid.max.y {
					grid.max.y = node.y;
				}
			}

			if empty {
				grid = NavGridData::default();
			}

			*nav_grid = Self {
				cells,
				cell_distance: GridCellDistance::inspect_field(map),
				method: TMethod::from(grid),
			}
		}
	}

	fn get_grid_positions(&self, start: Vec3, end: Vec3) -> (Vec3, Vec3) {
		(
			Vec3::new(start.x.round(), start.y.round(), start.z.round()),
			Vec3::new(end.x.round(), end.y.round(), end.z.round()),
		)
	}

	fn replace(item: Option<&mut Vec3>, replace: Vec3, compare: &Vec3) {
		let Some(item) = item else {
			return;
		};

		if item != compare {
			return;
		};

		*item = replace;
	}
}

impl<TMethod> ComputePath for NavGrid<TMethod>
where
	TMethod: ComputePath,
{
	fn compute_path(&self, start: Vec3, end: Vec3) -> Vec<Vec3> {
		let (start_on_grid, end_on_grid) = self.get_grid_positions(start, end);
		let mut path = self.method.compute_path(start_on_grid, end_on_grid);

		Self::replace(path.first_mut(), start, &start_on_grid);
		Self::replace(path.last_mut(), end, &end_on_grid);

		path
	}
}

#[derive(Debug, PartialEq, Default)]
pub(crate) struct NavGridData {
	pub(crate) min: NavGridNode,
	pub(crate) max: NavGridNode,
	pub(crate) obstacles: HashSet<NavGridNode>,
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{simple_init, test_tools::utils::SingleThreadedApp, traits::mock::Mock};
	use mockall::{mock, predicate::eq};

	#[derive(Debug, PartialEq, Default)]
	struct _Method(NavGridData);

	mock! {
		_Method {}
		impl ComputePath for _Method {
			fn compute_path(&self, start: Vec3, end: Vec3) -> Vec<Vec3>;
		}
	}

	simple_init!(Mock_Method);

	impl From<NavGridData> for _Method {
		fn from(data: NavGridData) -> Self {
			Self(data)
		}
	}

	#[derive(Component, Default)]
	struct _Map {
		cells: Vec<NavCell>,
		cell_distance: f32,
	}

	impl Iterate for _Map {
		type TItem<'a>
			= &'a NavCell
		where
			Self: 'a;

		fn iterate(&self) -> impl Iterator<Item = &'_ NavCell> {
			self.cells.iter()
		}
	}

	impl InspectAble<GridCellDistance> for _Map {
		fn get_inspect_able_field(&self) -> f32 {
			self.cell_distance
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, NavGrid::<_Method>::update_from::<_Map>);

		app
	}

	#[test]
	fn do_update() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				NavGrid::<_Method>::default(),
				_Map {
					cells: vec![
						NavCell {
							translation: Vec3::new(1., 0., 2.),
							is_walkable: true,
						},
						NavCell {
							translation: Vec3::new(2., 0., 1.),
							is_walkable: false,
						},
					],
					cell_distance: 11.,
				},
			))
			.id();

		app.update();

		assert_eq!(
			Some(&NavGrid {
				cells: vec![
					NavCell {
						translation: Vec3::new(1., 0., 2.),
						is_walkable: true,
					},
					NavCell {
						translation: Vec3::new(2., 0., 1.),
						is_walkable: false,
					},
				],
				cell_distance: 11.,
				method: _Method(NavGridData {
					min: NavGridNode { x: 1, y: 1 },
					max: NavGridNode { x: 2, y: 2 },
					obstacles: HashSet::from([NavGridNode { x: 2, y: 1 }]),
				})
			}),
			app.world().entity(entity).get::<NavGrid<_Method>>(),
		);
	}

	#[test]
	fn no_update_if_map_unchanged() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				NavGrid::<_Method>::default(),
				_Map {
					cells: vec![NavCell {
						translation: Vec3::new(1., 2., 3.),
						..default()
					}],
					cell_distance: 42.,
				},
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(NavGrid::<_Method>::default());
		app.update();

		assert_eq!(
			Some(&NavGrid::<_Method>::default()),
			app.world().entity(entity).get::<NavGrid<_Method>>(),
		);
	}

	#[test]
	fn update_again_if_map_changed() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				NavGrid::<_Method>::default(),
				_Map {
					cells: vec![NavCell {
						translation: Vec3::new(1., 2., 3.),
						is_walkable: false,
					}],
					cell_distance: 521.,
				},
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(NavGrid::<_Method>::default())
			.get_mut::<_Map>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some(&NavGrid {
				cells: vec![NavCell {
					translation: Vec3::new(1., 2., 3.),
					is_walkable: false,
				}],
				cell_distance: 521.,
				method: _Method(NavGridData {
					min: NavGridNode { x: 1, y: 3 },
					max: NavGridNode { x: 1, y: 3 },
					obstacles: HashSet::from([NavGridNode { x: 1, y: 3 }]),
				})
			}),
			app.world().entity(entity).get::<NavGrid<_Method>>(),
		);
	}

	#[test]
	fn grid_0_0_if_map_empty() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((NavGrid::<_Method>::default(), _Map::default()))
			.id();

		app.update();

		assert_eq!(
			Some(&NavGrid {
				cells: vec![],
				cell_distance: 0.,
				method: _Method(NavGridData {
					min: NavGridNode { x: 0, y: 0 },
					max: NavGridNode { x: 0, y: 0 },
					obstacles: HashSet::from([]),
				})
			}),
			app.world().entity(entity).get::<NavGrid<_Method>>(),
		);
	}

	#[test]
	fn call_compute_path_with_start_and_end() {
		let start = Vec3::new(1., 1., 1.);
		let end = Vec3::new(2., 2., 2.);
		let grid = NavGrid {
			method: Mock_Method::new_mock(|mock| {
				mock.expect_compute_path()
					.times(1)
					.with(eq(start), eq(end))
					.return_const(vec![]);
			}),
			cell_distance: 1.,
			cells: vec![],
		};

		grid.compute_path(start, end);
	}

	#[test]
	fn return_computed_path() {
		let path = [
			Vec3::new(1., 1., 1.),
			Vec3::new(2., 1., 1.),
			Vec3::new(3., 1., 1.),
		];
		let grid = NavGrid {
			method: Mock_Method::new_mock(|mock| {
				mock.expect_compute_path().return_const(path);
			}),
			cell_distance: 1.,
			cells: vec![],
		};

		let computed_path = grid.compute_path(path[0], path[2]);

		assert_eq!(Vec::from(path), computed_path);
	}

	#[test]
	fn call_compute_path_with_start_rounded() {
		let start = Vec3::new(1., 1., 1.);
		let end = Vec3::new(2., 2., 2.);
		let grid = NavGrid {
			method: Mock_Method::new_mock(|mock| {
				mock.expect_compute_path()
					.times(1)
					.with(eq(start), eq(end))
					.return_const(vec![]);
			}),
			cell_distance: 1.,
			cells: vec![],
		};

		grid.compute_path(Vec3::new(0.9, 1., 1.3), end);
	}

	#[test]
	fn call_compute_path_with_end_rounded() {
		let start = Vec3::new(1., 1., 1.);
		let end = Vec3::new(2., 2., 2.);
		let grid = NavGrid {
			method: Mock_Method::new_mock(|mock| {
				mock.expect_compute_path()
					.times(1)
					.with(eq(start), eq(end))
					.return_const(vec![]);
			}),
			cell_distance: 1.,
			cells: vec![],
		};

		grid.compute_path(start, Vec3::new(1.9, 2., 2.2));
	}

	#[test]
	fn replace_start_and_end_with_called_start_and_end() {
		let start = Vec3::new(1., 1., 1.);
		let end = Vec3::new(2., 2., 2.);
		let grid = NavGrid {
			method: Mock_Method::new_mock(|mock| {
				mock.expect_compute_path()
					.times(1)
					.with(eq(start), eq(end))
					.return_const(vec![
						start,
						Vec3::new(10., 1., 11.),
						Vec3::new(4., 1., 5.),
						end,
					]);
			}),
			cell_distance: 1.,
			cells: vec![],
		};

		let path = grid.compute_path(Vec3::new(0.8, 1., 1.3), Vec3::new(2.1, 2., 1.9));
		assert_eq!(
			vec![
				Vec3::new(0.8, 1., 1.3),
				Vec3::new(10., 1., 11.),
				Vec3::new(4., 1., 5.),
				Vec3::new(2.1, 2., 1.9)
			],
			path,
		);
	}

	#[test]
	fn do_not_replace_start_with_called_start_if_path_omitted_start() {
		let start = Vec3::new(1., 1., 1.);
		let end = Vec3::new(2., 2., 2.);
		let grid = NavGrid {
			method: Mock_Method::new_mock(|mock| {
				mock.expect_compute_path()
					.times(1)
					.with(eq(start), eq(end))
					.return_const(vec![Vec3::new(10., 1., 11.), Vec3::new(4., 1., 5.), end]);
			}),
			cell_distance: 1.,
			cells: vec![],
		};

		let path = grid.compute_path(Vec3::new(1.1, 1., 1.3), Vec3::new(2.1, 2., 1.9));
		assert_eq!(
			vec![
				Vec3::new(10., 1., 11.),
				Vec3::new(4., 1., 5.),
				Vec3::new(2.1, 2., 1.9)
			],
			path,
		);
	}

	#[test]
	fn do_not_replace_end_with_called_end_if_path_omitted_end() {
		let start = Vec3::new(1., 1., 1.);
		let end = Vec3::new(2., 2., 2.);
		let grid = NavGrid {
			method: Mock_Method::new_mock(|mock| {
				mock.expect_compute_path()
					.times(1)
					.with(eq(start), eq(end))
					.return_const(vec![start, Vec3::new(10., 1., 11.), Vec3::new(4., 1., 5.)]);
			}),
			cell_distance: 1.,
			cells: vec![],
		};

		let path = grid.compute_path(Vec3::new(1.1, 1., 1.3), Vec3::new(2.1, 2., 1.9));
		assert_eq!(
			vec![
				Vec3::new(1.1, 1., 1.3),
				Vec3::new(10., 1., 11.),
				Vec3::new(4., 1., 5.),
			],
			path,
		);
	}
}
