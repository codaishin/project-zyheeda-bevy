pub(crate) mod grid_context;
pub(crate) mod subdivide;

use crate::{
	line_wide::{LineNode, LineWide},
	traits::key_mapper::KeyMapper,
};
use bevy::prelude::*;
use common::{
	tools::Units,
	traits::handles_map_generation::{
		Graph,
		GraphLineOfSight,
		GraphNaivePath,
		GraphNode,
		GraphObstacle,
		GraphSuccessors,
		GraphTranslation,
		NaivePath,
	},
};
use grid_context::GridContext;
use std::{
	cmp::Ordering,
	collections::{HashMap, HashSet, hash_map::IntoIter},
	hash::{Hash, Hasher},
};

#[derive(Debug, PartialEq, Default, Clone)]
pub struct GridGraph<TValue = Vec3, TExtra = Obstacles, TGridContext = GridContext> {
	pub(crate) nodes: HashMap<(usize, usize), TValue>,
	pub(crate) extra: TExtra,
	pub(crate) context: TGridContext,
}

impl<TGridContext> GridGraph<Vec3, Obstacles, TGridContext>
where
	TGridContext: KeyMapper,
{
	fn is_straight(to: &GridGraphNode, origin_node: GridGraphNode) -> bool {
		origin_node.x() == to.x() || origin_node.z() == to.z()
	}

	fn path_border_offset(
		&self,
		origin_node: GridGraphNode,
		origin: Vec3,
		to: &GridGraphNode,
		half_width: Units,
	) -> Vec3 {
		let offset = origin - self.translation(&origin_node);

		match origin_node.x() == to.x() {
			true if offset.x > 0. => Vec3::new(offset.x + *half_width, 0., 0.),
			true => Vec3::new(offset.x - *half_width, 0., 0.),
			false if offset.z > 0. => Vec3::new(0., 0., offset.z + *half_width),
			false => Vec3::new(0., 0., offset.z - *half_width),
		}
	}

	fn path_step(origin_node: GridGraphNode, to: &GridGraphNode) -> (isize, isize) {
		match origin_node.x().cmp(&to.x()) {
			Ordering::Equal if origin_node.z() < to.z() => (0, 1),
			Ordering::Equal => (0, -1),
			Ordering::Less => (1, 0),
			Ordering::Greater => (-1, 0),
		}
	}
}

impl<TGridContext> Graph for GridGraph<Vec3, Obstacles, TGridContext>
where
	TGridContext: KeyMapper,
{
	type TNode = GridGraphNode;
}

impl<TGridContext> GraphNode for GridGraph<Vec3, Obstacles, TGridContext>
where
	TGridContext: KeyMapper,
{
	type TNNode = GridGraphNode;

	fn node(&self, translation: Vec3) -> Option<Self::TNNode> {
		let key = self.context.key_for(translation)?;

		if !self.nodes.contains_key(&key) {
			return None;
		}

		Some(GridGraphNode { key })
	}
}

impl<TGridContext> GraphSuccessors for GridGraph<Vec3, Obstacles, TGridContext>
where
	TGridContext: KeyMapper,
{
	type TSNode = GridGraphNode;

	fn successors(&self, node: &Self::TSNode) -> impl Iterator<Item = Self::TSNode> {
		NEIGHBORS.iter().filter_map(|(i, j)| {
			let key = (
				node.key.0.checked_add_signed(*i)?,
				node.key.1.checked_add_signed(*j)?,
			);

			if !self.nodes.contains_key(&key) {
				return None;
			}

			Some(GridGraphNode { key })
		})
	}
}

impl<TGridContext> GraphLineOfSight for GridGraph<Vec3, Obstacles, TGridContext>
where
	TGridContext: KeyMapper,
{
	type TLNode = GridGraphNode;

	fn line_of_sight(&self, a: &Self::TLNode, b: &Self::TLNode) -> bool {
		LineWide::new(a, b).all(|LineNode { x, z }| !self.extra.obstacles.contains(&(x, z)))
	}
}

impl<TGridContext> GraphTranslation for GridGraph<Vec3, Obstacles, TGridContext>
where
	TGridContext: KeyMapper,
{
	type TTNode = GridGraphNode;

	fn translation(&self, GridGraphNode { key }: &Self::TTNode) -> Vec3 {
		match self.nodes.get(key).copied() {
			Some(translation) => translation,
			None => unreachable!(
				"Tried retrieving translation of an invalid node, should not have happened. \
				 How was this node created?"
			),
		}
	}
}

impl<TGridContext> GraphObstacle for GridGraph<Vec3, Obstacles, TGridContext>
where
	TGridContext: KeyMapper,
{
	type TONode = GridGraphNode;

	fn is_obstacle(&self, GridGraphNode { key }: &Self::TONode) -> bool {
		self.extra.obstacles.contains(key)
	}
}

impl<TGridContext> GraphNaivePath for GridGraph<Vec3, Obstacles, TGridContext>
where
	TGridContext: KeyMapper,
{
	type TNNode = GridGraphNode;

	/// Compute a naive path with the following heuristics:
	///
	/// - diagonal path evaluation does a single on-grid los check
	///   - [`NaivePath::Ok`]: line of site okay
	///   - [`NaivePath::CannotCompute`]: line of site check failed
	///   - in practice this should always succeed when the used origin's node and target node
	///     are neighbors in a valid path
	/// - straight path evaluation attempts to ensure the agent would not collide with a non walkable
	///   or non existing cell
	///   - [`NaivePath::Ok`]: path is fully traversable
	///   - [`NaivePath::CannotCompute`]: path cannot be traversed
	///   - [`NaivePath::PartialUntil`]: the path cannot be traversed further than
	///     the provided vector
	///   - in practice usage of the partial path should be sufficient to avoid agent collisions
	///     when the used origin's node and target node are neighbors in a valid path
	fn naive_path(&self, origin: Vec3, to: &Self::TNNode, half_width: Units) -> NaivePath {
		let Some(origin_node) = self.node(origin) else {
			return NaivePath::CannotCompute;
		};

		if self.is_obstacle(&origin_node) {
			return NaivePath::CannotCompute;
		}

		if !Self::is_straight(to, origin_node) {
			return match self.line_of_sight(&origin_node, to) {
				true => NaivePath::Ok,
				false => NaivePath::CannotCompute,
			};
		}

		let path_border_offset = self.path_border_offset(origin_node, origin, to, half_width);
		let key_step = Self::path_step(origin_node, to);

		let mut furthest = origin_node;
		while &furthest != to {
			let mut next = furthest;

			let Some(key) = next.key.0.checked_add_signed(key_step.0) else {
				return NaivePath::PartialUntil(self.translation(&furthest));
			};
			next.key.0 = key;

			let Some(key) = next.key.1.checked_add_signed(key_step.1) else {
				return NaivePath::PartialUntil(self.translation(&furthest));
			};
			next.key.1 = key;

			if self.is_obstacle(&next) {
				return NaivePath::PartialUntil(self.translation(&furthest));
			}

			let Some(next_translation) = self.nodes.get(&next.key).copied() else {
				return NaivePath::CannotCompute;
			};

			let grazing = next_translation + path_border_offset;

			let Some(grazing) = self.node(grazing) else {
				return NaivePath::PartialUntil(self.translation(&furthest));
			};

			if self.is_obstacle(&grazing) {
				return NaivePath::PartialUntil(self.translation(&furthest));
			}

			furthest = next;
		}

		NaivePath::Ok
	}
}

impl<TValue, TExtra> IntoIterator for GridGraph<TValue, TExtra> {
	type Item = TValue;

	type IntoIter = Iter<TValue>;

	fn into_iter(self) -> Self::IntoIter {
		Iter {
			it: self.nodes.into_iter(),
		}
	}
}

pub struct Iter<TValue> {
	it: IntoIter<(usize, usize), TValue>,
}

impl<TValue> Iterator for Iter<TValue> {
	type Item = TValue;

	fn next(&mut self) -> Option<Self::Item> {
		let (.., value) = self.it.next()?;

		Some(value)
	}
}

const NEIGHBORS: &[(isize, isize)] = &[
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
	key: (usize, usize),
}

impl GridGraphNode {
	pub(crate) fn x(&self) -> usize {
		self.key.0
	}

	pub(crate) fn z(&self) -> usize {
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
	pub(crate) obstacles: HashSet<(usize, usize)>,
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::traits::clamp_zero_positive::ClampZeroPositive;
	use mockall::{mock, predicate::eq};
	use testing::{Mock, simple_init};

	mock! {
		_Mapper {}
		impl KeyMapper for _Mapper {
			fn key_for(&self, translation: Vec3) -> Option<(usize, usize)>;
		}
	}

	simple_init!(Mock_Mapper);

	#[test]
	fn gat_matching_node() {
		let graph = GridGraph {
			nodes: HashMap::from([((1, 2), Vec3::new(1., 2., 3.))]),
			extra: default(),
			context: Mock_Mapper::new_mock(|mock| {
				mock.expect_key_for().return_const((1, 2));
			}),
		};

		let node = graph.node(Vec3::default());

		assert_eq!(Some(GridGraphNode { key: (1, 2) }), node);
	}

	#[test]
	fn gat_matching_node_none() {
		let graph = GridGraph {
			nodes: HashMap::from([((1, 2), Vec3::new(1., 2., 3.))]),
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
			nodes: HashMap::from([((1, 2), Vec3::new(1., 2., 3.))]),
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
			nodes: HashMap::from([((1, 2), Vec3::default())]),
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
			nodes: HashMap::from([
				((1, 1), Vec3::default()),
				((1, 2), Vec3::default()),
				((1, 3), Vec3::default()),
				((2, 1), Vec3::default()),
				((2, 2), Vec3::default()),
				((2, 3), Vec3::default()),
				((3, 1), Vec3::default()),
				((3, 3), Vec3::default()),
				((3, 4), Vec3::default()),
				((1, 4), Vec3::default()),
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

	#[test]
	fn naive_path_okay_when_not_straight_and_no_obstacles_in_the_way() {
		let graph = GridGraph {
			nodes: HashMap::from([
				((1, 1), Vec3::new(1., 0., 1.)),
				((1, 2), Vec3::new(1., 0., 2.)),
				((1, 3), Vec3::new(1., 0., 3.)),
				((2, 1), Vec3::new(2., 0., 1.)),
				((2, 2), Vec3::new(2., 0., 2.)),
				((2, 3), Vec3::new(2., 0., 3.)),
			]),
			extra: Obstacles {
				obstacles: HashSet::from([]),
			},
			context: Mock_Mapper::new_mock(|mock| {
				mock.expect_key_for()
					.returning(|Vec3 { x, z, .. }| Some((x as usize, z as usize)));
			}),
		};

		let node = graph.node(Vec3::new(2., 0., 3.)).expect("NO SUCH NODE");
		let path = graph.naive_path(Vec3::new(1., 0., 1.), &node, Units::new(0.1));

		assert_eq!(NaivePath::Ok, path);
	}

	#[test]
	fn naive_path_not_computable_when_not_straight_and_obstacles_in_the_way() {
		let graph = GridGraph {
			nodes: HashMap::from([
				((1, 1), Vec3::new(1., 0., 1.)),
				((1, 2), Vec3::new(1., 0., 2.)),
				((1, 3), Vec3::new(1., 0., 3.)),
				((2, 1), Vec3::new(2., 0., 1.)),
				((2, 2), Vec3::new(2., 0., 2.)),
				((2, 3), Vec3::new(2., 0., 3.)),
			]),
			extra: Obstacles {
				obstacles: HashSet::from([(2, 1)]),
			},
			context: Mock_Mapper::new_mock(|mock| {
				mock.expect_key_for()
					.returning(|Vec3 { x, z, .. }| Some((x as usize, z as usize)));
			}),
		};

		let node = graph.node(Vec3::new(2., 0., 3.)).expect("NO SUCH NODE");
		let path = graph.naive_path(Vec3::new(1., 0., 1.), &node, Units::new(0.1));

		assert_eq!(NaivePath::CannotCompute, path);
	}

	#[test]
	fn naive_path_not_computable_when_origin_node_cannot_be_retrieved() {
		let graph = GridGraph {
			nodes: HashMap::from([
				((1, 2), Vec3::new(1., 0., 2.)),
				((1, 3), Vec3::new(1., 0., 3.)),
				((2, 1), Vec3::new(2., 0., 1.)),
				((2, 2), Vec3::new(2., 0., 2.)),
				((2, 3), Vec3::new(2., 0., 3.)),
			]),
			extra: Obstacles {
				obstacles: HashSet::from([]),
			},
			context: Mock_Mapper::new_mock(|mock| {
				mock.expect_key_for()
					.returning(|Vec3 { x, z, .. }| Some((x as usize, z as usize)));
			}),
		};

		let node = graph.node(Vec3::new(2., 0., 3.)).expect("NO SUCH NODE");
		let path = graph.naive_path(Vec3::new(1., 0., 1.), &node, Units::new(0.1));

		assert_eq!(NaivePath::CannotCompute, path);
	}

	#[test]
	fn naive_path_partial_when_straight_but_obstructed_because_width_grazed_obstacle() {
		let graph = GridGraph {
			nodes: HashMap::from([
				((1, 1), Vec3::new(1., 0., 1.)),
				((1, 2), Vec3::new(1., 0., 2.)),
				((1, 3), Vec3::new(1., 0., 3.)),
				((2, 1), Vec3::new(2., 0., 1.)),
				((2, 2), Vec3::new(2., 0., 2.)),
				((2, 3), Vec3::new(2., 0., 3.)),
			]),
			extra: Obstacles {
				obstacles: HashSet::from([(2, 3)]),
			},
			context: Mock_Mapper::new_mock(|mock| {
				mock.expect_key_for()
					.returning(|Vec3 { x, z, .. }| Some((x.round() as usize, z.round() as usize)));
			}),
		};

		let node = graph.node(Vec3::new(1., 0., 3.)).expect("NO SUCH NODE");
		let path = graph.naive_path(Vec3::new(1.3, 0., 1.), &node, Units::new(0.3));

		assert_eq!(NaivePath::PartialUntil(Vec3::new(1., 0., 2.)), path);
	}

	#[test]
	fn naive_path_partial_when_straight_but_obstructed_because_width_grazed_non_existing() {
		let graph = GridGraph {
			nodes: HashMap::from([
				((1, 1), Vec3::new(1., 0., 1.)),
				((1, 2), Vec3::new(1., 0., 2.)),
				((1, 3), Vec3::new(1., 0., 3.)),
				((2, 1), Vec3::new(2., 0., 1.)),
				((2, 2), Vec3::new(2., 0., 2.)),
			]),
			extra: Obstacles {
				obstacles: HashSet::from([]),
			},
			context: Mock_Mapper::new_mock(|mock| {
				mock.expect_key_for()
					.returning(|Vec3 { x, z, .. }| Some((x.round() as usize, z.round() as usize)));
			}),
		};

		let node = graph.node(Vec3::new(1., 0., 3.)).expect("NO SUCH NODE");
		let path = graph.naive_path(Vec3::new(1.3, 0., 1.), &node, Units::new(0.3));

		assert_eq!(NaivePath::PartialUntil(Vec3::new(1., 0., 2.)), path);
	}

	#[test]
	fn naive_path_non_computable_when_straight_but_in_between_node_does_not_exist() {
		let graph = GridGraph {
			nodes: HashMap::from([
				((1, 1), Vec3::new(1., 0., 1.)),
				((1, 3), Vec3::new(1., 0., 3.)),
			]),
			extra: Obstacles {
				obstacles: HashSet::from([]),
			},
			context: Mock_Mapper::new_mock(|mock| {
				mock.expect_key_for()
					.returning(|Vec3 { x, z, .. }| Some((x.round() as usize, z.round() as usize)));
			}),
		};

		let node = graph.node(Vec3::new(1., 0., 3.)).expect("NO SUCH NODE");
		let path = graph.naive_path(Vec3::new(1., 0., 1.), &node, Units::new(0.3));

		assert_eq!(NaivePath::CannotCompute, path);
	}

	#[test]
	fn naive_path_partial_when_straight_but_obstructed_because_width_grazed_obstacle_other_side() {
		let graph = GridGraph {
			nodes: HashMap::from([
				((1, 1), Vec3::new(1., 0., 1.)),
				((1, 2), Vec3::new(1., 0., 2.)),
				((1, 3), Vec3::new(1., 0., 3.)),
				((2, 1), Vec3::new(2., 0., 1.)),
				((2, 2), Vec3::new(2., 0., 2.)),
				((2, 3), Vec3::new(2., 0., 3.)),
			]),
			extra: Obstacles {
				obstacles: HashSet::from([(1, 3)]),
			},
			context: Mock_Mapper::new_mock(|mock| {
				mock.expect_key_for()
					.returning(|Vec3 { x, z, .. }| Some((x.round() as usize, z.round() as usize)));
			}),
		};

		let node = graph.node(Vec3::new(2., 0., 3.)).expect("NO SUCH NODE");
		let path = graph.naive_path(Vec3::new(1.7, 0., 1.), &node, Units::new(0.3));

		assert_eq!(NaivePath::PartialUntil(Vec3::new(2., 0., 2.)), path);
	}

	#[test]
	fn naive_path_partial_when_straight_but_obstructed_because_width_grazed_obstacle_rotated() {
		let graph = GridGraph {
			nodes: HashMap::from([
				((1, 1), Vec3::new(1., 0., 1.)),
				((1, 2), Vec3::new(1., 0., 2.)),
				((2, 1), Vec3::new(2., 0., 1.)),
				((2, 2), Vec3::new(2., 0., 2.)),
				((3, 1), Vec3::new(3., 0., 1.)),
				((3, 2), Vec3::new(3., 0., 2.)),
			]),
			extra: Obstacles {
				obstacles: HashSet::from([(3, 2)]),
			},
			context: Mock_Mapper::new_mock(|mock| {
				mock.expect_key_for()
					.returning(|Vec3 { x, z, .. }| Some((x.round() as usize, z.round() as usize)));
			}),
		};

		let node = graph.node(Vec3::new(3., 0., 1.)).expect("NO SUCH NODE");
		let path = graph.naive_path(Vec3::new(1., 0., 1.3), &node, Units::new(0.3));

		assert_eq!(NaivePath::PartialUntil(Vec3::new(2., 0., 1.)), path);
	}

	#[test]
	fn naive_path_partial_when_straight_but_obstructed_because_width_grazed_obstacle_rotated_other_side()
	 {
		let graph = GridGraph {
			nodes: HashMap::from([
				((1, 1), Vec3::new(1., 0., 1.)),
				((1, 2), Vec3::new(1., 0., 2.)),
				((2, 1), Vec3::new(2., 0., 1.)),
				((2, 2), Vec3::new(2., 0., 2.)),
				((3, 1), Vec3::new(3., 0., 1.)),
				((3, 2), Vec3::new(3., 0., 2.)),
			]),
			extra: Obstacles {
				obstacles: HashSet::from([(3, 1)]),
			},
			context: Mock_Mapper::new_mock(|mock| {
				mock.expect_key_for()
					.returning(|Vec3 { x, z, .. }| Some((x.round() as usize, z.round() as usize)));
			}),
		};

		let node = graph.node(Vec3::new(3., 0., 2.)).expect("NO SUCH NODE");
		let path = graph.naive_path(Vec3::new(1., 0., 1.7), &node, Units::new(0.3));

		assert_eq!(NaivePath::PartialUntil(Vec3::new(2., 0., 2.)), path);
	}

	#[test]
	fn naive_path_partial_when_straight_but_obstructed_because_width_grazed_obstacle_to_negative() {
		let graph = GridGraph {
			nodes: HashMap::from([
				((1, 1), Vec3::new(1., 0., 1.)),
				((1, 2), Vec3::new(1., 0., 2.)),
				((1, 3), Vec3::new(1., 0., 3.)),
				((2, 1), Vec3::new(2., 0., 1.)),
				((2, 2), Vec3::new(2., 0., 2.)),
				((2, 3), Vec3::new(2., 0., 3.)),
			]),
			extra: Obstacles {
				obstacles: HashSet::from([(2, 1)]),
			},
			context: Mock_Mapper::new_mock(|mock| {
				mock.expect_key_for()
					.returning(|Vec3 { x, z, .. }| Some((x.round() as usize, z.round() as usize)));
			}),
		};

		let node = graph.node(Vec3::new(1., 0., 1.)).expect("NO SUCH NODE");
		let path = graph.naive_path(Vec3::new(1.3, 0., 3.), &node, Units::new(0.3));

		assert_eq!(NaivePath::PartialUntil(Vec3::new(1., 0., 2.)), path);
	}

	#[test]
	fn naive_path_partial_when_straight_but_obstructed_because_width_grazed_obstacle_rotated_negative()
	 {
		let graph = GridGraph {
			nodes: HashMap::from([
				((1, 1), Vec3::new(1., 0., 1.)),
				((1, 2), Vec3::new(1., 0., 2.)),
				((2, 1), Vec3::new(2., 0., 1.)),
				((2, 2), Vec3::new(2., 0., 2.)),
				((3, 1), Vec3::new(3., 0., 1.)),
				((3, 2), Vec3::new(3., 0., 2.)),
			]),
			extra: Obstacles {
				obstacles: HashSet::from([(1, 2)]),
			},
			context: Mock_Mapper::new_mock(|mock| {
				mock.expect_key_for()
					.returning(|Vec3 { x, z, .. }| Some((x.round() as usize, z.round() as usize)));
			}),
		};

		let node = graph.node(Vec3::new(1., 0., 1.)).expect("NO SUCH NODE");
		let path = graph.naive_path(Vec3::new(3., 0., 1.3), &node, Units::new(0.3));

		assert_eq!(NaivePath::PartialUntil(Vec3::new(2., 0., 1.)), path);
	}

	#[test]
	fn naive_path_ok_when_straight_and_width_grazing_no_obstructed_nodes() {
		let graph = GridGraph {
			nodes: HashMap::from([
				((1, 1), Vec3::new(1., 0., 1.)),
				((1, 2), Vec3::new(1., 0., 2.)),
				((1, 3), Vec3::new(1., 0., 3.)),
				((2, 1), Vec3::new(2., 0., 1.)),
				((2, 2), Vec3::new(2., 0., 2.)),
				((2, 3), Vec3::new(2., 0., 3.)),
			]),
			extra: Obstacles {
				obstacles: HashSet::from([]),
			},
			context: Mock_Mapper::new_mock(|mock| {
				mock.expect_key_for()
					.returning(|Vec3 { x, z, .. }| Some((x.round() as usize, z.round() as usize)));
			}),
		};

		let node = graph.node(Vec3::new(1., 0., 3.)).expect("NO SUCH NODE");
		let path = graph.naive_path(Vec3::new(1., 0., 1.), &node, Units::new(0.3));

		assert_eq!(NaivePath::Ok, path);
	}

	#[test]
	fn naive_path_partial_when_straight_and_obstacles_in_the_way() {
		let graph = GridGraph {
			nodes: HashMap::from([
				((1, 1), Vec3::new(1., 0., 1.)),
				((1, 2), Vec3::new(1., 0., 2.)),
				((1, 3), Vec3::new(1., 0., 3.)),
				((2, 1), Vec3::new(2., 0., 1.)),
				((2, 2), Vec3::new(2., 0., 2.)),
				((2, 3), Vec3::new(2., 0., 3.)),
			]),
			extra: Obstacles {
				obstacles: HashSet::from([(1, 3)]),
			},
			context: Mock_Mapper::new_mock(|mock| {
				mock.expect_key_for()
					.returning(|Vec3 { x, z, .. }| Some((x.round() as usize, z.round() as usize)));
			}),
		};

		let node = graph.node(Vec3::new(1., 0., 3.)).expect("NO SUCH NODE");
		let path = graph.naive_path(Vec3::new(1.4, 0., 1.), &node, Units::new(0.3));

		assert_eq!(NaivePath::PartialUntil(Vec3::new(1.0, 0.0, 2.0)), path);
	}

	#[test]
	fn naive_path_not_computable_when_straight_and_start_is_obstacle() {
		let graph = GridGraph {
			nodes: HashMap::from([
				((1, 1), Vec3::new(1., 0., 1.)),
				((1, 2), Vec3::new(1., 0., 2.)),
				((1, 3), Vec3::new(1., 0., 3.)),
			]),
			extra: Obstacles {
				obstacles: HashSet::from([(1, 1)]),
			},
			context: Mock_Mapper::new_mock(|mock| {
				mock.expect_key_for()
					.returning(|Vec3 { x, z, .. }| Some((x.round() as usize, z.round() as usize)));
			}),
		};

		let node = graph.node(Vec3::new(1., 0., 3.)).expect("NO SUCH NODE");
		let path = graph.naive_path(Vec3::new(1., 0., 1.), &node, Units::new(0.1));

		assert_eq!(NaivePath::CannotCompute, path);
	}
}
