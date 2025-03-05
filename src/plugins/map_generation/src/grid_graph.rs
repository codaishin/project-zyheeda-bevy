pub(crate) mod grid_context;

use crate::{
	line_wide::{LineNode, LineWide},
	traits::{grid_cell_distance::GridCellDistance, key_mapper::KeyMapper},
};
use bevy::prelude::*;
use common::{
	tools::Units,
	traits::handles_map_generation::{
		Graph,
		GraphClamp,
		GraphLineOfSight,
		GraphNode,
		GraphObstacle,
		GraphSuccessors,
		GraphTranslation,
	},
};
use grid_context::GridContext;
use std::{
	collections::{HashMap, HashSet, hash_map::IntoIter},
	hash::{Hash, Hasher},
};

#[derive(Debug, PartialEq, Default, Clone)]
pub struct GridGraph<TValue = GridCell, TExtra = Obstacles, TGridContext = GridContext> {
	pub(crate) cells: HashMap<(i32, i32), TValue>,
	pub(crate) extra: TExtra,
	pub(crate) context: TGridContext,
}

impl<TGridContext> Graph for GridGraph<GridCell, Obstacles, TGridContext>
where
	TGridContext: KeyMapper + GridCellDistance,
{
	type TNode = GridGraphNode;
}

impl<TGridContext> GraphNode for GridGraph<GridCell, Obstacles, TGridContext>
where
	TGridContext: KeyMapper,
{
	type TNNode = GridGraphNode;

	fn node(&self, translation: Vec3) -> Option<Self::TNNode> {
		let key = self.context.key_for(translation);

		if !self.cells.contains_key(&key) {
			return None;
		}

		Some(GridGraphNode { key })
	}
}

impl<TGridContext> GraphSuccessors for GridGraph<GridCell, Obstacles, TGridContext>
where
	TGridContext: KeyMapper,
{
	type TSNode = GridGraphNode;

	fn successors(&self, node: &Self::TSNode) -> impl Iterator<Item = Self::TSNode> {
		NEIGHBORS.iter().filter_map(|(i, j)| {
			let key = (node.key.0 + i, node.key.1 + j);

			if !self.cells.contains_key(&key) {
				return None;
			}

			Some(GridGraphNode { key })
		})
	}
}

impl<TGridContext> GraphLineOfSight for GridGraph<GridCell, Obstacles, TGridContext>
where
	TGridContext: KeyMapper,
{
	type TLNode = GridGraphNode;

	fn line_of_sight(&self, a: &Self::TLNode, b: &Self::TLNode) -> bool {
		LineWide::new(a, b).all(|LineNode { x, z }| !self.extra.obstacles.contains(&(x, z)))
	}
}

impl<TGridContext> GraphTranslation for GridGraph<GridCell, Obstacles, TGridContext>
where
	TGridContext: KeyMapper,
{
	type TTNode = GridGraphNode;

	fn translation(&self, GridGraphNode { key }: &Self::TTNode) -> Vec3 {
		match self.cells.get(key).map(|n| n.value) {
			Some(translation) => translation,
			None => unreachable!(
				"Tried retrieving translation of an invalid node, should not have happened. \
				 How was this node created?"
			),
		}
	}
}

impl<TGridContext> GraphObstacle for GridGraph<GridCell, Obstacles, TGridContext>
where
	TGridContext: KeyMapper,
{
	type TONode = GridGraphNode;

	fn is_obstacle(&self, GridGraphNode { key }: &Self::TONode) -> bool {
		self.extra.obstacles.contains(key)
	}
}

impl<TGridContext> GraphClamp for GridGraph<GridCell, Obstacles, TGridContext>
where
	TGridContext: KeyMapper + GridCellDistance,
{
	fn clamp(&self, translation: Vec3, agent_radius: Units) -> Option<Vec3> {
		let key = self.context.key_for(translation);
		let cell = self.cells.get(&key)?;
		let center = cell.value;
		let mut offset = translation - center;
		let signum_x = signum(offset.x);
		let signum_z = signum(offset.z);
		let quadrants = Quadrant::from_direction(signum_x as i32, signum_z as i32);

		if !cell.borders_obstacles(&quadrants) {
			return Some(translation);
		}

		let max_offset = self.context.grid_cell_distance() as f32 / 2. - *agent_radius;

		offset.x = f32::min(max_offset, offset.x.abs()) * signum_x;
		offset.z = f32::min(max_offset, offset.z.abs()) * signum_z;

		Some(center + offset)
	}
}

impl<TValue, TExtra> IntoIterator for GridGraph<TValue, TExtra> {
	type Item = TValue;
	type IntoIter = Iter<TValue>;

	fn into_iter(self) -> Self::IntoIter {
		Iter {
			it: self.cells.into_iter(),
		}
	}
}

pub struct Iter<TValue> {
	it: IntoIter<(i32, i32), TValue>,
}

impl<TValue> Iterator for Iter<TValue> {
	type Item = TValue;

	fn next(&mut self) -> Option<Self::Item> {
		let (.., next) = self.it.next()?;

		Some(next)
	}
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct GridCell<TValue = Vec3> {
	pub(crate) value: TValue,
	pub(crate) obstacle_quadrants: HashSet<Quadrant>,
}

impl GridCell {
	pub(crate) fn borders_obstacles(&self, quadrants: &HashSet<Quadrant>) -> bool {
		self.obstacle_quadrants
			.intersection(quadrants)
			.next()
			.is_some()
	}
}

#[cfg(test)]
impl<TValue> GridCell<TValue> {
	pub(crate) fn new(value: TValue) -> Self {
		Self {
			value,
			obstacle_quadrants: HashSet::from([]),
		}
	}
	pub(crate) fn bordering_obstacles<const N: usize>(mut self, obstacles: [Quadrant; N]) -> Self {
		self.obstacle_quadrants = HashSet::from(obstacles);

		self
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) enum Quadrant {
	NegXNegZ,
	NegXPosZ,
	PosXNegZ,
	PosXPosZ,
}

impl Quadrant {
	pub(crate) fn from_direction(x: i32, z: i32) -> HashSet<Quadrant> {
		match (x, z) {
			(x, z) if x < 0 && z < 0 => HashSet::from([Quadrant::NegXNegZ]),
			(x, z) if x < 0 && z > 0 => HashSet::from([Quadrant::NegXPosZ]),
			(x, z) if x > 0 && z < 0 => HashSet::from([Quadrant::PosXNegZ]),
			(x, z) if x > 0 && z > 0 => HashSet::from([Quadrant::PosXPosZ]),
			(x, _) if x < 0 => HashSet::from([Quadrant::NegXNegZ, Quadrant::NegXPosZ]),
			(x, _) if x > 0 => HashSet::from([Quadrant::PosXNegZ, Quadrant::PosXPosZ]),
			(_, z) if z < 0 => HashSet::from([Quadrant::NegXNegZ, Quadrant::PosXNegZ]),
			(_, z) if z > 0 => HashSet::from([Quadrant::NegXPosZ, Quadrant::PosXPosZ]),
			_ => HashSet::new(),
		}
	}
}

pub(crate) const NEIGHBORS: [(i32, i32); 8] = [
	(-1, -1),
	(-1, 0),
	(-1, 1),
	(0, -1),
	(0, 1),
	(1, -1),
	(1, 0),
	(1, 1),
];

#[derive(Debug, Clone, Copy)]
pub struct GridGraphNode {
	key: (i32, i32),
}

impl GridGraphNode {
	pub(crate) fn x(&self) -> i32 {
		self.key.0
	}

	pub(crate) fn z(&self) -> i32 {
		self.key.1
	}
}

impl PartialEq for GridGraphNode {
	fn eq(&self, other: &Self) -> bool {
		self.key == other.key
	}
}

impl Eq for GridGraphNode {}

impl Hash for GridGraphNode {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.key.hash(state);
	}
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct Obstacles {
	pub(crate) obstacles: HashSet<(i32, i32)>,
}

fn signum(v: f32) -> f32 {
	if v == 0. {
		return 0.;
	}

	v.signum()
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		simple_init,
		traits::{clamp_zero_positive::ClampZeroPositive, mock::Mock},
	};
	use mockall::{mock, predicate::eq};
	use test_case::test_case;

	mock! {
		_Mapper {}
		impl KeyMapper for _Mapper {
			fn key_for(&self, translation: Vec3) -> (i32, i32);
		}
		impl GridCellDistance for _Mapper {
				fn grid_cell_distance(&self) -> u8;
		}
	}

	simple_init!(Mock_Mapper);

	#[test]
	fn gat_matching_node() {
		let graph = GridGraph {
			cells: HashMap::from([(
				(1, 2),
				GridCell {
					value: Vec3::new(1., 2., 3.),
					..default()
				},
			)]),
			extra: default(),
			context: Mock_Mapper::new_mock(|mock| {
				mock.expect_key_for().return_const((1, 2));
			}),
		};

		let node = graph.node(Vec3::default());

		assert_eq!(Some(GridGraphNode { key: (1, 2) }), node);
	}

	#[test]
	fn get_matching_node_none() {
		let graph = GridGraph {
			cells: HashMap::from([(
				(1, 2),
				GridCell {
					value: Vec3::new(1., 2., 3.),
					..default()
				},
			)]),
			extra: default(),
			context: Mock_Mapper::new_mock(|mock| {
				mock.expect_key_for().return_const((1, 3));
			}),
		};

		let node = graph.node(Vec3::default());

		assert_eq!(None, node);
	}

	#[test]
	fn supply_key_getter_with_proper_arguments() {
		let graph = GridGraph {
			cells: HashMap::from([(
				(1, 2),
				GridCell {
					value: Vec3::new(1., 2., 3.),
					..default()
				},
			)]),
			extra: default(),
			context: Mock_Mapper::new_mock(|mock| {
				mock.expect_key_for()
					.times(1)
					.with(eq(Vec3::new(7., 8., 9.)))
					.return_const((0, 0));
			}),
		};

		graph.node(Vec3::new(7., 8., 9.));
	}

	#[test]
	fn node_is_obstacle() {
		let graph = GridGraph {
			cells: HashMap::from([((1, 2), GridCell::default())]),
			extra: Obstacles {
				obstacles: HashSet::from([(1, 2)]),
			},
			context: Mock_Mapper::new_mock(|mock| {
				mock.expect_key_for().return_const((1, 2));
			}),
		};

		let node = graph.node(Vec3::default()).expect("NO NODE RETURNED");
		let is_obstacle = graph.is_obstacle(&node);

		assert!(is_obstacle);
	}

	#[test]
	fn get_neighbors() {
		let graph = GridGraph {
			cells: HashMap::from([
				((1, 1), GridCell::default()),
				((1, 2), GridCell::default()),
				((1, 3), GridCell::default()),
				((2, 1), GridCell::default()),
				((2, 2), GridCell::default()),
				((2, 3), GridCell::default()),
				((3, 1), GridCell::default()),
				((3, 3), GridCell::default()),
				((3, 4), GridCell::default()),
				((1, 4), GridCell::default()),
			]),
			extra: Obstacles {
				obstacles: HashSet::from([(1, 2), (3, 1)]),
			},
			context: Mock_Mapper::new_mock(|mock| {
				mock.expect_key_for().return_const((2, 2));
			}),
		};

		let node = graph.node(Vec3::default()).expect("NO NODE RETURNED");
		let successors = graph.successors(&node);

		assert_eq!(
			vec![
				GridGraphNode { key: (1, 1) },
				GridGraphNode { key: (1, 2) },
				GridGraphNode { key: (1, 3) },
				GridGraphNode { key: (2, 1) },
				GridGraphNode { key: (2, 3) },
				GridGraphNode { key: (3, 1) },
				GridGraphNode { key: (3, 3) },
			],
			successors.collect::<Vec<_>>()
		);
	}

	fn cell<const N: usize>(x: f32, z: f32, obstacles: [Quadrant; N]) -> GridCell {
		GridCell {
			value: Vec3::new(x, 0., z),
			obstacle_quadrants: HashSet::from(obstacles),
		}
	}

	fn vec3(x: f32, z: f32) -> Vec3 {
		Vec3::new(x, 0., z)
	}

	#[test_case(cell(0., 0., []), 1, vec3(0.0, 0.0), 0.1, vec3(0.0, 0.0); "matching cell")]
	#[test_case(cell(0., 0., []), 1, vec3(1.3, 1.2), 0.1, vec3(1.3, 1.2); "no obstacle")]
	#[test_case(cell(1., 2., [Quadrant::PosXPosZ]), 1, vec3(1.4, 2.0), 0.4, vec3(1.1, 2.0); "limited x")]
	#[test_case(cell(1., 2., [Quadrant::NegXPosZ]), 1, vec3(1.4, 2.0), 0.4, vec3(1.4, 2.0); "not limited x")]
	#[test_case(cell(1., 2., [Quadrant::PosXPosZ]), 1, vec3(1.0, 2.4), 0.4, vec3(1.0, 2.1); "limited z")]
	#[test_case(cell(1., 2., [Quadrant::PosXNegZ]), 1, vec3(1.0, 2.4), 0.4, vec3(1.0, 2.4); "not limited z")]
	#[test_case(cell(1., 2., [Quadrant::NegXNegZ]), 1, vec3(0.7, 2.0), 0.4, vec3(0.9, 2.0); "limited neg x")]
	#[test_case(cell(1., 2., [Quadrant::NegXNegZ]), 1, vec3(1.0, 1.7), 0.4, vec3(1.0, 1.9); "limited neg z")]
	#[test_case(cell(1., 2., [Quadrant::PosXPosZ]), 2, vec3(1.9, 2.0), 0.4, vec3(1.6, 2.0); "limited x with cell distance 2")]
	fn clamp_translation(
		cell: GridCell,
		cell_distance: u8,
		translation: Vec3,
		radius: f32,
		expected_result: Vec3,
	) {
		let graph = GridGraph {
			cells: HashMap::from([((0, 0), cell)]),
			extra: default(),
			context: Mock_Mapper::new_mock(|mock| {
				mock.expect_key_for()
					.with(eq(translation))
					.return_const((0, 0));
				mock.expect_grid_cell_distance().return_const(cell_distance);
			}),
		};

		let result = graph.clamp(translation, Units::new(radius));

		assert_eq!(Some(expected_result), result);
	}
}
