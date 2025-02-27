use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

type TGetKeyFn = fn(&HashMap<(i32, i32), Vec3>, Vec3) -> Option<(i32, i32)>;

#[derive(Debug, PartialEq, Clone)]
pub struct Graph {
	pub(crate) translations: HashMap<(i32, i32), Vec3>,
	pub(crate) obstacles: HashSet<(i32, i32)>,
	pub(crate) get_key: TGetKeyFn,
}

impl Graph {
	pub(crate) fn get_node(&self, translation: Vec3) -> Option<GraphNode> {
		let key = (self.get_key)(&self.translations, translation)?;
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

	#[test]
	fn gat_matching_node() {
		let graph = Graph {
			translations: HashMap::from([((1, 2), Vec3::new(1., 2., 3.))]),
			obstacles: default(),
			get_key: |_, _| Some((1, 2)),
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
			get_key: |map, vec| {
				assert_eq!(
					(
						&HashMap::from([((1, 2), Vec3::new(1., 2., 3.))]),
						Vec3::new(7., 8., 9.)
					),
					(map, vec)
				);
				None
			},
		};

		graph.get_node(Vec3::new(7., 8., 9.));
	}

	#[test]
	fn node_is_obstacle() {
		let graph = Graph {
			translations: HashMap::from([((1, 2), Vec3::default())]),
			obstacles: HashSet::from([(1, 2)]),
			get_key: |_, _| Some((1, 2)),
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
			get_key: |_, _| Some((2, 2)),
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
