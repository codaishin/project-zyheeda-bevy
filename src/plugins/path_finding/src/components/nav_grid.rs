use crate::{tools::nav_grid_node::NavGridNode, traits::compute_path_lazy::ComputePathLazy};
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
			let cell_distance = GridCellDistance::inspect_field(map);
			let mut empty = true;
			let mut grid = NavGridData {
				min: NavGridNode::MAX,
				max: NavGridNode::MIN,
				obstacles: HashSet::default(),
			};

			for cell in map.iterate() {
				let translation = cell.translation / cell_distance;
				let node = NavGridNode {
					x: translation.x as i32,
					y: translation.z as i32,
				};

				empty = false;

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
				cell_distance,
				method: TMethod::from(grid),
			}
		}
	}

	fn nav_grid_node(&self, mut value: Vec3) -> Result<NavGridNode, CellDistanceZero> {
		if self.cell_distance == 0. {
			return Err(CellDistanceZero);
		}

		value /= self.cell_distance;

		Ok(NavGridNode {
			x: value.x.round() as i32,
			y: value.z.round() as i32,
		})
	}

	fn replace(path_item: Option<&mut Vec3>, replace: Vec3) {
		let Some(path_item) = path_item else {
			return;
		};
		let replace_rounded = Vec3::new(replace.x.round(), replace.y.round(), replace.z.round());

		if path_item != &replace_rounded {
			return;
		};

		*path_item = replace;
	}
}

impl<TMethod> ComputePath for NavGrid<TMethod>
where
	TMethod: ComputePathLazy,
{
	type TError = CellDistanceZero;

	fn compute_path(&self, start: Vec3, end: Vec3) -> Result<Vec<Vec3>, Self::TError> {
		let start_node = self.nav_grid_node(start)?;
		let end_node = self.nav_grid_node(end)?;
		let mut path = self
			.method
			.compute_path(start_node, end_node)
			.map(|n| Vec3::new(n.x as f32, 0., n.y as f32) * self.cell_distance)
			.collect::<Vec<_>>();

		Self::replace(path.first_mut(), start);
		Self::replace(path.last_mut(), end);

		Ok(path)
	}
}

#[derive(Debug, PartialEq)]
pub struct CellDistanceZero;

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
		impl ComputePathLazy for _Method {
			fn compute_path(&self, start: NavGridNode, end: NavGridNode) -> impl Iterator<Item = NavGridNode>;
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
							translation: Vec3::new(2., 0., 4.),
							is_walkable: true,
						},
						NavCell {
							translation: Vec3::new(4., 0., 2.),
							is_walkable: false,
						},
					],
					cell_distance: 2.,
				},
			))
			.id();

		app.update();

		assert_eq!(
			Some(&NavGrid {
				cell_distance: 2.,
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
					cell_distance: 1.,
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
					cell_distance: 1.,
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
				cell_distance: 1.,
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
					.with(
						eq(NavGridNode { x: 1, y: 1 }),
						eq(NavGridNode { x: 2, y: 2 }),
					)
					.returning(|_, _| Box::new([].into_iter()));
			}),
			cell_distance: 1.,
		};

		_ = grid.compute_path(start, end);
	}

	#[test]
	fn return_computed_path() {
		let path = [
			NavGridNode { x: 1, y: 1 },
			NavGridNode { x: 2, y: 2 },
			NavGridNode { x: 3, y: 3 },
		];
		let grid = NavGrid {
			method: Mock_Method::new_mock(|mock| {
				mock.expect_compute_path()
					.returning(move |_, _| Box::new(path.into_iter()));
			}),
			cell_distance: 1.,
		};

		let computed_path = grid.compute_path(Vec3::new(1., 0., 1.), Vec3::new(3., 0., 3.));

		assert_eq!(
			Ok(Vec::from(
				path.map(|n| Vec3::new(n.x as f32, 0., n.y as f32))
			)),
			computed_path
		);
	}

	#[test]
	fn call_compute_path_with_start_rounded() {
		let grid = NavGrid {
			method: Mock_Method::new_mock(|mock| {
				mock.expect_compute_path()
					.times(1)
					.with(
						eq(NavGridNode { x: 1, y: 1 }),
						eq(NavGridNode { x: 2, y: 2 }),
					)
					.returning(|_, _| Box::new([].into_iter()));
			}),
			cell_distance: 1.,
		};

		_ = grid.compute_path(Vec3::new(0.9, 0., 1.3), Vec3::new(2., 0., 2.));
	}

	#[test]
	fn call_compute_path_with_end_rounded() {
		let grid = NavGrid {
			method: Mock_Method::new_mock(|mock| {
				mock.expect_compute_path()
					.times(1)
					.with(
						eq(NavGridNode { x: 1, y: 1 }),
						eq(NavGridNode { x: 2, y: 2 }),
					)
					.returning(|_, _| Box::new([].into_iter()));
			}),
			cell_distance: 1.,
		};

		_ = grid.compute_path(Vec3::new(1., 0., 1.), Vec3::new(1.9, 0., 2.2));
	}

	#[test]
	fn replace_start_and_end_with_called_start_and_end() {
		let grid = NavGrid {
			method: Mock_Method::new_mock(|mock| {
				mock.expect_compute_path()
					.times(1)
					.with(
						eq(NavGridNode { x: 1, y: 1 }),
						eq(NavGridNode { x: 2, y: 2 }),
					)
					.returning(|_, _| {
						Box::new(
							[
								NavGridNode { x: 1, y: 1 },
								NavGridNode { x: 10, y: 11 },
								NavGridNode { x: 4, y: 5 },
								NavGridNode { x: 2, y: 2 },
							]
							.into_iter(),
						)
					});
			}),
			cell_distance: 1.,
		};

		let path = grid.compute_path(Vec3::new(0.8, 0., 1.3), Vec3::new(2.1, 0., 1.9));
		assert_eq!(
			Ok(vec![
				Vec3::new(0.8, 0., 1.3),
				Vec3::new(10., 0., 11.),
				Vec3::new(4., 0., 5.),
				Vec3::new(2.1, 0., 1.9)
			]),
			path,
		);
	}

	#[test]
	fn do_not_replace_start_with_called_start_if_path_omitted_start() {
		let grid = NavGrid {
			method: Mock_Method::new_mock(|mock| {
				mock.expect_compute_path()
					.times(1)
					.with(
						eq(NavGridNode { x: 1, y: 1 }),
						eq(NavGridNode { x: 2, y: 2 }),
					)
					.returning(|_, _| {
						Box::new(
							[
								NavGridNode { x: 10, y: 11 },
								NavGridNode { x: 4, y: 5 },
								NavGridNode { x: 2, y: 2 },
							]
							.into_iter(),
						)
					});
			}),
			cell_distance: 1.,
		};

		let path = grid.compute_path(Vec3::new(1.1, 0., 1.3), Vec3::new(2.1, 0., 1.9));
		assert_eq!(
			Ok(vec![
				Vec3::new(10., 0., 11.),
				Vec3::new(4., 0., 5.),
				Vec3::new(2.1, 0., 1.9)
			]),
			path,
		);
	}

	#[test]
	fn do_not_replace_end_with_called_end_if_path_omitted_end() {
		let grid = NavGrid {
			method: Mock_Method::new_mock(|mock| {
				mock.expect_compute_path()
					.times(1)
					.with(
						eq(NavGridNode { x: 1, y: 1 }),
						eq(NavGridNode { x: 2, y: 2 }),
					)
					.returning(|_, _| {
						Box::new(
							[
								NavGridNode { x: 1, y: 1 },
								NavGridNode { x: 10, y: 11 },
								NavGridNode { x: 4, y: 5 },
							]
							.into_iter(),
						)
					});
			}),
			cell_distance: 1.,
		};

		let path = grid.compute_path(Vec3::new(1.1, 0., 1.3), Vec3::new(2.1, 0., 1.9));
		assert_eq!(
			Ok(vec![
				Vec3::new(1.1, 0., 1.3),
				Vec3::new(10., 0., 11.),
				Vec3::new(4., 0., 5.),
			]),
			path,
		);
	}

	#[test]
	fn call_compute_path_with_start_end_rounded_scaled() {
		let grid = NavGrid {
			method: Mock_Method::new_mock(|mock| {
				mock.expect_compute_path()
					.times(1)
					.with(
						eq(NavGridNode { x: 1, y: 1 }),
						eq(NavGridNode { x: 2, y: 2 }),
					)
					.returning(|_, _| Box::new([].into_iter()));
			}),
			cell_distance: 2.,
		};

		_ = grid.compute_path(Vec3::new(1.9, 0., 2.3), Vec3::new(3.9, 0., 4.3));
	}

	#[test]
	fn call_compute_path_with_start_end_scaled_before_rounded() {
		let grid = NavGrid {
			method: Mock_Method::new_mock(|mock| {
				mock.expect_compute_path()
					.times(1)
					.with(
						eq(NavGridNode { x: 1, y: 1 }),
						eq(NavGridNode { x: 2, y: 2 }),
					)
					.returning(|_, _| Box::new([].into_iter()));
			}),
			cell_distance: 0.5,
		};

		_ = grid.compute_path(Vec3::new(0.4, 0.5, 0.6), Vec3::new(0.9, 1., 1.1));
	}

	#[test]
	fn return_path_scaled_back_via_grid_distance() {
		let grid = NavGrid {
			method: Mock_Method::new_mock(|mock| {
				mock.expect_compute_path()
					.times(1)
					.with(
						eq(NavGridNode { x: 1, y: 1 }),
						eq(NavGridNode { x: 2, y: 2 }),
					)
					.returning(|_, _| {
						Box::new(
							[
								NavGridNode { x: 1, y: 1 },
								NavGridNode { x: 10, y: 11 },
								NavGridNode { x: 4, y: 5 },
								NavGridNode { x: 2, y: 2 },
							]
							.into_iter(),
						)
					});
			}),
			cell_distance: 2.,
		};

		let path = grid.compute_path(Vec3::new(1.9, 0., 2.3), Vec3::new(3.9, 0., 4.3));
		assert_eq!(
			Ok(vec![
				Vec3::new(1.9, 0., 2.3),
				Vec3::new(20., 0., 22.),
				Vec3::new(8., 0., 10.),
				Vec3::new(3.9, 0., 4.3)
			]),
			path,
		);
	}

	#[test]
	fn return_cell_distance_zero_error() {
		let grid = NavGrid {
			method: Mock_Method::new_mock(|mock| {
				mock.expect_compute_path()
					.returning(|_, _| Box::new([].into_iter()));
			}),
			cell_distance: 0.,
		};

		let path = grid.compute_path(Vec3::new(1.9, 0., 2.3), Vec3::new(3.9, 0., 4.3));
		assert_eq!(Err(CellDistanceZero), path,);
	}
}
