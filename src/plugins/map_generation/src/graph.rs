pub(crate) mod grid_context;

use crate::traits::key_mapper::KeyMapper;
use bevy::prelude::*;
use grid_context::GridContext;
use std::collections::{HashMap, HashSet};

#[derive(Debug, PartialEq, Clone)]
pub struct Graph<TValue = Vec3, TKeyMapper = GridContext> {
	pub(crate) translations: HashMap<(i32, i32), TValue>,
	pub(crate) obstacles: HashSet<(i32, i32)>,
	pub(crate) key_mapper: TKeyMapper,
}

impl<TKeyMapper> Graph<Vec3, TKeyMapper>
where
	TKeyMapper: KeyMapper,
{
	pub(crate) fn get_node(&self, translation: Vec3) -> Option<GraphNode> {
		let key = self.key_mapper.key_for(translation);
		let translation = *self.translations.get(&key)?;
		let obstacle = self.obstacles.contains(&key);

		Some(GraphNode {
			key,
			obstacle,
			translation,
		})
	}

	pub(crate) fn get_neighbors<'a>(
		&'a self,
		node: &'a GraphNode,
	) -> impl Iterator<Item = GraphNode> + 'a {
		NEIGHBORS.iter().filter_map(|(i, j)| {
			let key = (node.key.0 + i, node.key.1 + j);
			let translation = *self.translations.get(&key)?;
			let obstacle = self.obstacles.contains(&key);

			Some(GraphNode {
				key,
				obstacle,
				translation,
			})
		})
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct GraphNode {
	key: (i32, i32),
	obstacle: bool,
	translation: Vec3,
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
		let graph = Graph {
			translations: HashMap::from([((1, 2), Vec3::new(1., 2., 3.))]),
			obstacles: default(),
			key_mapper: Mock_Mapper::new_mock(|mock| {
				mock.expect_key_for().return_const((1, 2));
			}),
		};

		let node = graph.get_node(Vec3::default());

		assert_eq!(
			Some(GraphNode {
				key: (1, 2),
				translation: Vec3::new(1., 2., 3.),
				obstacle: false,
			}),
			node
		);
	}

	#[test]
	fn supply_key_getter_with_proper_arguments() {
		let graph = Graph {
			translations: HashMap::from([((1, 2), Vec3::new(1., 2., 3.))]),
			obstacles: default(),
			key_mapper: Mock_Mapper::new_mock(|mock| {
				mock.expect_key_for()
					.times(1)
					.with(eq(Vec3::new(7., 8., 9.)))
					.return_const((0, 0));
			}),
		};

		graph.get_node(Vec3::new(7., 8., 9.));
	}

	#[test]
	fn node_is_obstacle() {
		let graph = Graph {
			translations: HashMap::from([((1, 2), Vec3::default())]),
			obstacles: HashSet::from([(1, 2)]),
			key_mapper: Mock_Mapper::new_mock(|mock| {
				mock.expect_key_for().return_const((1, 2));
			}),
		};

		let node = graph.get_node(Vec3::default());

		assert_eq!(
			Some(GraphNode {
				key: (1, 2),
				translation: Vec3::default(),
				obstacle: true,
			}),
			node
		);
	}

	#[test]
	fn get_neighbors() {
		let graph = Graph {
			translations: HashMap::from([
				((1, 1), Vec3::new(1., 1., 1.)),
				((1, 2), Vec3::new(2., 1., 1.)),
				((1, 3), Vec3::new(3., 1., 1.)),
				((2, 1), Vec3::new(4., 1., 1.)),
				((2, 2), Vec3::new(5., 1., 1.)),
				((2, 3), Vec3::new(6., 1., 1.)),
				((3, 1), Vec3::new(7., 1., 1.)),
				((3, 2), Vec3::new(8., 1., 1.)),
				((3, 3), Vec3::new(9., 1., 1.)),
				((3, 4), Vec3::new(-1., 1., 1.)),
				((1, 4), Vec3::new(-1., 1., 1.)),
			]),
			obstacles: HashSet::from([(1, 2), (3, 1)]),
			key_mapper: Mock_Mapper::new_mock(|mock| {
				mock.expect_key_for().return_const((2, 2));
			}),
		};

		let node = graph.get_node(Vec3::default()).expect("NO NODE RETURNED");

		assert_eq!(
			vec![
				GraphNode {
					key: (1, 1),
					translation: Vec3::new(1., 1., 1.),
					obstacle: false,
				},
				GraphNode {
					key: (1, 2),
					translation: Vec3::new(2., 1., 1.),
					obstacle: true,
				},
				GraphNode {
					key: (1, 3),
					translation: Vec3::new(3., 1., 1.),
					obstacle: false,
				},
				GraphNode {
					key: (2, 1),
					translation: Vec3::new(4., 1., 1.),
					obstacle: false,
				},
				GraphNode {
					key: (2, 3),
					translation: Vec3::new(6., 1., 1.),
					obstacle: false,
				},
				GraphNode {
					key: (3, 1),
					translation: Vec3::new(7., 1., 1.),
					obstacle: true,
				},
				GraphNode {
					key: (3, 2),
					translation: Vec3::new(8., 1., 1.),
					obstacle: false,
				},
				GraphNode {
					key: (3, 3),
					translation: Vec3::new(9., 1., 1.),
					obstacle: false,
				},
			],
			graph.get_neighbors(&node).collect::<Vec<_>>()
		);
	}
}
