pub(crate) mod grid_context;

use crate::{
	line_wide::{LineNode, LineWide},
	traits::key_mapper::KeyMapper,
};
use bevy::prelude::*;
use common::traits::handles_map_generation::{
	Graph,
	GraphLineOfSight,
	GraphNode,
	GraphObstacle,
	GraphSuccessors,
	GraphTranslation,
};
use grid_context::GridContext;
use std::{
	collections::{HashMap, HashSet, hash_map::IntoIter},
	hash::{Hash, Hasher},
};

#[derive(Debug, PartialEq, Default, Clone)]
pub struct GridGraph<TValue = Vec3, TExtra = Obstacles, TGridContext = GridContext> {
	pub(crate) nodes: HashMap<(i32, i32), TValue>,
	pub(crate) extra: TExtra,
	pub(crate) context: TGridContext,
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
		let key = self.context.key_for(translation);

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
			let key = (node.key.0 + i, node.key.1 + j);

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
	it: IntoIter<(i32, i32), TValue>,
}

impl<TValue> Iterator for Iter<TValue> {
	type Item = TValue;

	fn next(&mut self) -> Option<Self::Item> {
		let (.., value) = self.it.next()?;

		Some(value)
	}
}

const NEIGHBORS: &[(i32, i32)] = &[
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

#[cfg(test)]
mod tests {
	use super::*;
	use common::{simple_init, traits::mock::Mock};
	use mockall::{mock, predicate::eq};

	mock! {
		_Mapper {}
		impl KeyMapper for _Mapper {
			fn key_for(&self, translation: Vec3) -> (i32, i32);
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
}
