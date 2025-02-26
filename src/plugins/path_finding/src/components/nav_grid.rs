use crate::tools::nav_grid_node::NavGridNode;
use bevy::prelude::*;
use common::traits::{
	handles_map_generation::NavCell,
	handles_path_finding::ComputePath,
	iterate::Iterate,
	thread_safe::ThreadSafe,
};
use std::collections::HashSet;

#[derive(Component, Debug, PartialEq, Default)]
pub struct NavGrid<TMethod> {
	pub(crate) method: TMethod,
}

impl<TMethod> NavGrid<TMethod> {
	pub(crate) fn update_from<TMap>(mut maps: Query<(&TMap, &mut Self), Changed<TMap>>)
	where
		for<'a> TMap: Component + Iterate<TItem<'a> = &'a NavCell>,
		TMethod: ThreadSafe + From<NavGridData>,
	{
		for (map, mut nav_grid) in &mut maps {
			let mut empty = true;
			let mut grid = NavGridData {
				min: NavGridNode::MAX,
				max: NavGridNode::MIN,
				obstacles: HashSet::default(),
			};

			for cell in map.iterate() {
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
				method: TMethod::from(grid),
			}
		}
	}
}

impl<TMethod> ComputePath for NavGrid<TMethod>
where
	TMethod: ComputePath,
{
	fn compute_path(&self, start: Vec3, end: Vec3) -> Vec<Vec3> {
		self.method.compute_path(start, end)
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
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Component)]
	struct _Map(Vec<NavCell>);

	#[derive(Debug, PartialEq, Default)]
	struct _Method(NavGridData);

	impl From<NavGridData> for _Method {
		fn from(data: NavGridData) -> Self {
			Self(data)
		}
	}

	impl Iterate for _Map {
		type TItem<'a>
			= &'a NavCell
		where
			Self: 'a;

		fn iterate(&self) -> impl Iterator<Item = &'_ NavCell> {
			self.0.iter()
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
				_Map(vec![
					NavCell {
						translation: Vec3::new(1., 0., 2.),
						is_walkable: true,
					},
					NavCell {
						translation: Vec3::new(2., 0., 1.),
						is_walkable: false,
					},
				]),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&NavGrid {
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
				_Map(vec![NavCell {
					translation: Vec3::new(1., 2., 3.),
					..default()
				}]),
			))
			.id();

		app.update();
		app.world_mut().entity_mut(entity).insert(NavGrid {
			method: _Method::default(),
		});
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
				_Map(vec![NavCell {
					translation: Vec3::new(1., 2., 3.),
					is_walkable: false,
				}]),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(NavGrid {
				method: _Method::default(),
			})
			.get_mut::<_Map>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some(&NavGrid {
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
			.spawn((NavGrid::<_Method>::default(), _Map(vec![])))
			.id();

		app.update();

		assert_eq!(
			Some(&NavGrid {
				method: _Method(NavGridData {
					min: NavGridNode { x: 0, y: 0 },
					max: NavGridNode { x: 0, y: 0 },
					obstacles: HashSet::from([]),
				})
			}),
			app.world().entity(entity).get::<NavGrid<_Method>>(),
		);
	}
}
