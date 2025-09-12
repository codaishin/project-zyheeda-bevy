use crate::traits::compute_path_lazy::ComputePathLazy;
use bevy::prelude::*;
use common::{
	errors::{Error, Level},
	tools::Units,
	traits::{
		handles_map_generation::{Graph, NaivePath},
		handles_path_finding::ComputePath,
		register_derived_component::{DerivableFrom, InsertDerivedComponent},
		thread_safe::ThreadSafe,
	},
};

#[derive(Component, Debug, PartialEq, Default)]
pub struct Navigation<TMethod, TGraph> {
	pub(crate) graph: TGraph,
	pub(crate) method: TMethod,
}

impl<TMethod, TGraph> Navigation<TMethod, TGraph>
where
	TMethod: ComputePathLazy<TGraph>,
	TGraph: Graph,
{
	fn replace_start(
		&self,
		node: &TGraph::TNode,
		path: &[TGraph::TNode],
		start_node: TGraph::TNode,
		start: Vec3,
		agent_radius: Units,
	) -> Option<Vec<Vec3>> {
		if node != &start_node {
			return None;
		}

		let [_, next, ..] = path else {
			return None;
		};

		Some(match self.graph.naive_path(start, next, agent_radius) {
			NaivePath::Ok => vec![start],
			NaivePath::PartialUntil(extra) => vec![start, extra],
			NaivePath::CannotCompute => vec![self.graph.translation(node)],
		})
	}

	fn replace_end(
		&self,
		node: &TGraph::TNode,
		path: &[TGraph::TNode],
		end_node: TGraph::TNode,
		end: Vec3,
		agent_radius: Units,
	) -> Option<Vec<Vec3>> {
		if node != &end_node {
			return None;
		};

		let [.., next, _] = path else {
			return None;
		};

		Some(match self.graph.naive_path(end, next, agent_radius) {
			NaivePath::Ok => vec![end],
			NaivePath::PartialUntil(extra) => vec![extra, end],
			NaivePath::CannotCompute => vec![self.graph.translation(node)],
		})
	}
}

impl<'w, 's, TMap, TMethod, TGraph> DerivableFrom<'w, 's, TMap> for Navigation<TMethod, TGraph>
where
	for<'a> TGraph: From<&'a TMap> + ThreadSafe,
	TMethod: Default + ThreadSafe,
{
	const INSERT: InsertDerivedComponent = InsertDerivedComponent::IfNew;

	type TParam = ();

	fn derive_from(_: Entity, map: &TMap, _: &()) -> Option<Self> {
		Some(Self {
			graph: TGraph::from(map),
			method: TMethod::default(),
		})
	}
}

impl<TMethod, TGraph> ComputePath for Navigation<TMethod, TGraph>
where
	TMethod: ComputePathLazy<TGraph>,
	TGraph: Graph,
{
	fn compute_path(&self, start: Vec3, end: Vec3, agent_radius: Units) -> Option<Vec<Vec3>> {
		let start_node = self.graph.node(start)?;
		let end_node = self.graph.node(end)?;

		if start_node == end_node {
			return Some(vec![start, end]);
		}

		let path = self
			.method
			.compute_path(&self.graph, start_node, end_node)
			.collect::<Vec<_>>();

		let path = path
			.iter()
			.flat_map(|node| {
				let new_start = self.replace_start(node, &path, start_node, start, agent_radius);
				if let Some(nodes) = new_start {
					return nodes;
				}

				let new_end = self.replace_end(node, &path, end_node, end, agent_radius);
				if let Some(nodes) = new_end {
					return nodes;
				}

				vec![self.graph.translation(node)]
			})
			.collect::<Vec<_>>();

		Some(path)
	}
}

#[derive(Debug, PartialEq)]
pub enum NavGridError {
	Empty,
	CellDistanceZero,
}

impl From<NavGridError> for Error {
	fn from(error: NavGridError) -> Self {
		match error {
			NavGridError::Empty => Error::Single {
				msg: "Source map is empty".to_owned(),
				lvl: Level::Error,
			},
			NavGridError::CellDistanceZero => Error::Single {
				msg: "`NavMap` cell distance is zero".to_owned(),
				lvl: Level::Error,
			},
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::traits::handles_map_generation::{
		GraphLineOfSight,
		GraphNaivePath,
		GraphNode,
		GraphObstacle,
		GraphSuccessors,
		GraphTranslation,
		NaivePath,
	};
	use macros::simple_mock;
	use mockall::predicate::eq;
	use testing::Mock;

	simple_mock! {
		_Method {}
		impl ComputePathLazy<_Graph> for _Method {
			fn compute_path(
				&self,
				graph: & _Graph,
				start: _Node,
				end: _Node,
			) -> impl Iterator<Item = _Node>;
		}
	}

	simple_mock! {
		_Method2 {}
		impl ComputePathLazy<Mock_Graph> for _Method2 {
			fn compute_path(
				&self,
				graph: & Mock_Graph,
				start: _Node,
				end: _Node,
			) -> impl Iterator<Item = _Node>;
		}
	}

	#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
	struct _Node(u8, u8, u8);

	#[derive(Debug, PartialEq)]
	struct _Graph;

	impl Graph for _Graph {
		type TNode = _Node;
	}

	impl GraphNode for _Graph {
		type TNNode = _Node;
		fn node(&self, Vec3 { x, y, z }: Vec3) -> Option<_Node> {
			Some(_Node(x.round() as u8, y.round() as u8, z.round() as u8))
		}
	}

	impl GraphTranslation for _Graph {
		type TTNode = _Node;
		fn translation(&self, _Node(x, y, z): &_Node) -> Vec3 {
			Vec3::new(*x as f32, *y as f32, *z as f32)
		}
	}

	impl GraphSuccessors for _Graph {
		type TSNode = _Node;
		fn successors(&self, _: &_Node) -> impl Iterator<Item = _Node> {
			[].into_iter()
		}
	}

	impl GraphLineOfSight for _Graph {
		type TLNode = _Node;
		fn line_of_sight(&self, _: &_Node, _: &_Node) -> bool {
			todo!()
		}
	}

	impl GraphObstacle for _Graph {
		type TONode = _Node;
		fn is_obstacle(&self, _: &_Node) -> bool {
			todo!()
		}
	}

	impl GraphNaivePath for _Graph {
		type TNNode = _Node;

		fn naive_path(&self, _: Vec3, _: &Self::TNNode, _: Units) -> NaivePath {
			todo!()
		}
	}

	simple_mock! {
		_Graph {}
		impl GraphNode for _Graph {
			type TNNode = _Node;
			fn node(&self, translation: Vec3) -> Option<_Node>;
		}
		impl GraphTranslation for _Graph {
			type TTNode = _Node;
			fn translation(&self, node: &_Node) -> Vec3;
		}
		impl GraphSuccessors for _Graph {
			type TSNode = _Node;
			fn successors(&self, node: &_Node) -> impl Iterator<Item = _Node>;
		}
		impl GraphLineOfSight for _Graph {
			type TLNode = _Node;
			fn line_of_sight(&self, a: &_Node, b: &_Node) -> bool;
		}
		impl GraphObstacle for _Graph {
			type TONode = _Node;
			fn is_obstacle(&self, node: &_Node) -> bool;
		}
		impl GraphNaivePath for _Graph {
			type TNNode = _Node;
			fn naive_path(&self, origin: Vec3, to: &_Node, width: Units) -> NaivePath;
		}
	}

	impl Graph for Mock_Graph {
		type TNode = _Node;
	}

	#[test]
	fn call_compute_path_with_start_and_end() {
		let start = Vec3::new(1., 1., 1.);
		let end = Vec3::new(2., 2., 2.);
		let grid = Navigation {
			method: Mock_Method::new_mock(|mock| {
				mock.expect_compute_path()
					.times(1)
					.with(eq(_Graph), eq(_Node(1, 1, 1)), eq(_Node(2, 2, 2)))
					.returning(|_, _, _| Box::new([].into_iter()));
			}),
			graph: _Graph,
		};

		_ = grid.compute_path(start, end, Units::from(0.1));
	}

	#[test]
	fn return_computed_path() {
		let path = [_Node(1, 1, 1), _Node(2, 2, 2), _Node(3, 3, 3)];
		let grid = Navigation {
			method: Mock_Method::new_mock(|mock| {
				mock.expect_compute_path()
					.returning(move |_, _, _| Box::new(path.into_iter()));
			}),
			graph: _Graph,
		};

		let computed_path = grid.compute_path(
			Vec3::new(1., 0., 1.),
			Vec3::new(3., 0., 3.),
			Units::from(0.1),
		);

		assert_eq!(
			Some(Vec::from(path.map(|_Node(x, y, z)| Vec3::new(
				x as f32, y as f32, z as f32
			)))),
			computed_path
		);
	}

	#[test]
	fn no_computation_when_start_and_end_on_same_node() {
		let grid = Navigation {
			method: Mock_Method::new_mock(|mock| {
				mock.expect_compute_path()
					.never()
					.returning(|_, _, _| Box::new([].into_iter()));
			}),
			graph: _Graph,
		};

		let path = grid.compute_path(
			Vec3::new(0.8, 1., 1.3),
			Vec3::new(1.1, 1., 0.9),
			Units::from(0.1),
		);
		assert_eq!(
			Some(vec![Vec3::new(0.8, 1., 1.3), Vec3::new(1.1, 1., 0.9)]),
			path
		);
	}

	#[test]
	fn replace_start_and_end_with_called_start_and_end() {
		let grid = Navigation {
			method: Mock_Method2::new_mock(|mock| {
				mock.expect_compute_path().returning(|_, _, _| {
					Box::new(
						[
							_Node(1, 1, 1),
							_Node(10, 10, 10),
							_Node(4, 4, 4),
							_Node(2, 2, 2),
						]
						.into_iter(),
					)
				});
			}),
			graph: Mock_Graph::new_mock(|mock| {
				mock.expect_node()
					.with(eq(Vec3::new(0.8, 1., 1.3)))
					.return_const(Some(_Node(1, 1, 1)));
				mock.expect_node()
					.with(eq(Vec3::new(2.1, 2., 1.9)))
					.return_const(Some(_Node(2, 2, 2)));
				mock.expect_translation()
					.returning(|_Node(x, y, z)| Vec3::new(*x as f32, *y as f32, *z as f32));
				mock.expect_naive_path().return_const(NaivePath::Ok);
			}),
		};

		let path = grid.compute_path(
			Vec3::new(0.8, 1., 1.3),
			Vec3::new(2.1, 2., 1.9),
			Units::from(0.1),
		);
		assert_eq!(
			Some(vec![
				Vec3::new(0.8, 1., 1.3),
				Vec3::new(10., 10., 10.),
				Vec3::new(4., 4., 4.),
				Vec3::new(2.1, 2., 1.9)
			]),
			path,
		);
	}

	#[test]
	fn do_not_replace_start_with_called_start_if_path_omitted_start() {
		let grid = Navigation {
			method: Mock_Method2::new_mock(|mock| {
				mock.expect_compute_path().returning(|_, _, _| {
					Box::new([_Node(10, 10, 10), _Node(4, 4, 4), _Node(2, 2, 2)].into_iter())
				});
			}),
			graph: Mock_Graph::new_mock(|mock| {
				mock.expect_node()
					.with(eq(Vec3::new(0.8, 1., 1.3)))
					.return_const(Some(_Node(1, 1, 1)));
				mock.expect_node()
					.with(eq(Vec3::new(2.1, 2., 1.9)))
					.return_const(Some(_Node(2, 2, 2)));
				mock.expect_translation()
					.returning(|_Node(x, y, z)| Vec3::new(*x as f32, *y as f32, *z as f32));
				mock.expect_naive_path().return_const(NaivePath::Ok);
			}),
		};

		let path = grid.compute_path(
			Vec3::new(0.8, 1., 1.3),
			Vec3::new(2.1, 2., 1.9),
			Units::from(0.1),
		);
		assert_eq!(
			Some(vec![
				Vec3::new(10., 10., 10.),
				Vec3::new(4., 4., 4.),
				Vec3::new(2.1, 2., 1.9)
			]),
			path,
		);
	}

	#[test]
	fn do_not_replace_end_with_called_end_if_path_omitted_end() {
		let grid = Navigation {
			method: Mock_Method2::new_mock(|mock| {
				mock.expect_compute_path().returning(|_, _, _| {
					Box::new([_Node(1, 1, 1), _Node(10, 10, 10), _Node(4, 4, 4)].into_iter())
				});
			}),
			graph: Mock_Graph::new_mock(|mock| {
				mock.expect_node()
					.with(eq(Vec3::new(0.8, 1., 1.3)))
					.return_const(Some(_Node(1, 1, 1)));
				mock.expect_node()
					.with(eq(Vec3::new(2.1, 2., 1.9)))
					.return_const(Some(_Node(2, 2, 2)));
				mock.expect_translation()
					.returning(|_Node(x, y, z)| Vec3::new(*x as f32, *y as f32, *z as f32));
				mock.expect_naive_path().return_const(NaivePath::Ok);
			}),
		};

		let path = grid.compute_path(
			Vec3::new(0.8, 1., 1.3),
			Vec3::new(2.1, 2., 1.9),
			Units::from(0.1),
		);
		assert_eq!(
			Some(vec![
				Vec3::new(0.8, 1., 1.3),
				Vec3::new(10., 10., 10.),
				Vec3::new(4., 4., 4.),
			]),
			path,
		);
	}

	#[test]
	fn replace_start_and_end_with_called_start_and_end_with_different_grid_mapping() {
		let grid = Navigation {
			method: Mock_Method2::new_mock(|mock| {
				mock.expect_compute_path().returning(|_, _, _| {
					Box::new(
						[
							_Node(1, 1, 1),
							_Node(10, 10, 10),
							_Node(4, 4, 4),
							_Node(2, 2, 2),
						]
						.into_iter(),
					)
				});
			}),
			graph: Mock_Graph::new_mock(|mock| {
				mock.expect_node()
					.with(eq(Vec3::new(0.8, 1., 1.3)))
					.return_const(Some(_Node(1, 1, 1)));
				mock.expect_node()
					.with(eq(Vec3::new(2.1, 2., 1.9)))
					.return_const(Some(_Node(2, 2, 2)));
				mock.expect_translation().returning(|_Node(x, y, z)| {
					Vec3::new(*x as f32 + 0.5, *y as f32 + 0.5, *z as f32 + 0.5)
				});
				mock.expect_naive_path().return_const(NaivePath::Ok);
			}),
		};

		let path = grid.compute_path(
			Vec3::new(0.8, 1., 1.3),
			Vec3::new(2.1, 2., 1.9),
			Units::from(0.1),
		);
		assert_eq!(
			Some(vec![
				Vec3::new(0.8, 1., 1.3),
				Vec3::new(10.5, 10.5, 10.5),
				Vec3::new(4.5, 4.5, 4.5),
				Vec3::new(2.1, 2., 1.9)
			]),
			path,
		);
	}

	#[test]
	fn replace_start_and_end_with_naive_path_corrected_start_and_end() {
		let grid = Navigation {
			method: Mock_Method2::new_mock(|mock| {
				mock.expect_compute_path().returning(|_, _, _| {
					Box::new(
						[
							_Node(1, 1, 1),
							_Node(10, 10, 10),
							_Node(4, 4, 4),
							_Node(2, 2, 2),
						]
						.into_iter(),
					)
				});
			}),
			graph: Mock_Graph::new_mock(|mock| {
				mock.expect_node()
					.with(eq(Vec3::new(0.8, 1., 1.3)))
					.return_const(Some(_Node(1, 1, 1)));
				mock.expect_node()
					.with(eq(Vec3::new(2.1, 2., 1.9)))
					.return_const(Some(_Node(2, 2, 2)));
				mock.expect_translation()
					.returning(|_Node(x, y, z)| Vec3::new(*x as f32, *y as f32, *z as f32));
				mock.expect_naive_path()
					.with(
						eq(Vec3::new(0.8, 1., 1.3)),
						eq(_Node(10, 10, 10)),
						eq(Units::from(0.1)),
					)
					.return_const(NaivePath::PartialUntil(Vec3::new(5., 5., 5.)));
				mock.expect_naive_path()
					.with(
						eq(Vec3::new(2.1, 2., 1.9)),
						eq(_Node(4, 4, 4)),
						eq(Units::from(0.1)),
					)
					.return_const(NaivePath::PartialUntil(Vec3::new(6., 6., 6.)));
			}),
		};

		let path = grid.compute_path(
			Vec3::new(0.8, 1., 1.3),
			Vec3::new(2.1, 2., 1.9),
			Units::from(0.1),
		);
		assert_eq!(
			Some(vec![
				Vec3::new(0.8, 1., 1.3),
				Vec3::new(5., 5., 5.),
				Vec3::new(10., 10., 10.),
				Vec3::new(4., 4., 4.),
				Vec3::new(6., 6., 6.),
				Vec3::new(2.1, 2., 1.9)
			]),
			path,
		);
	}

	#[test]
	fn do_not_replace_start_and_end_when_naive_path_cannot_be_computed() {
		let grid = Navigation {
			method: Mock_Method2::new_mock(|mock| {
				mock.expect_compute_path().returning(|_, _, _| {
					Box::new(
						[
							_Node(1, 1, 1),
							_Node(10, 10, 10),
							_Node(4, 4, 4),
							_Node(2, 2, 2),
						]
						.into_iter(),
					)
				});
			}),
			graph: Mock_Graph::new_mock(|mock| {
				mock.expect_node()
					.with(eq(Vec3::new(0.8, 1., 1.3)))
					.return_const(Some(_Node(1, 1, 1)));
				mock.expect_node()
					.with(eq(Vec3::new(2.1, 2., 1.9)))
					.return_const(Some(_Node(2, 2, 2)));
				mock.expect_translation()
					.returning(|_Node(x, y, z)| Vec3::new(*x as f32, *y as f32, *z as f32));
				mock.expect_naive_path()
					.return_const(NaivePath::CannotCompute);
			}),
		};

		let path = grid.compute_path(
			Vec3::new(0.8, 1., 1.3),
			Vec3::new(2.1, 2., 1.9),
			Units::from(0.1),
		);
		assert_eq!(
			Some(vec![
				Vec3::new(1., 1., 1.),
				Vec3::new(10., 10., 10.),
				Vec3::new(4., 4., 4.),
				Vec3::new(2., 2., 2.),
			]),
			path,
		);
	}

	#[test]
	fn do_not_replace_start_and_end_when_path_only_one_item_matching_start() {
		let grid = Navigation {
			method: Mock_Method2::new_mock(|mock| {
				mock.expect_compute_path()
					.returning(|_, _, _| Box::new([_Node(1, 1, 1)].into_iter()));
			}),
			graph: Mock_Graph::new_mock(|mock| {
				mock.expect_node()
					.with(eq(Vec3::new(0.8, 1., 1.3)))
					.return_const(Some(_Node(1, 1, 1)));
				mock.expect_node()
					.with(eq(Vec3::new(2.1, 2., 1.9)))
					.return_const(Some(_Node(2, 2, 2)));
				mock.expect_translation()
					.returning(|_Node(x, y, z)| Vec3::new(*x as f32, *y as f32, *z as f32));
				mock.expect_naive_path().never().return_const(NaivePath::Ok);
			}),
		};

		let path = grid.compute_path(
			Vec3::new(0.8, 1., 1.3),
			Vec3::new(2.1, 2., 1.9),
			Units::from(0.1),
		);
		assert_eq!(Some(vec![Vec3::new(1., 1., 1.)]), path);
	}

	#[test]
	fn do_not_replace_start_and_end_when_path_only_one_item_matching_end() {
		let grid = Navigation {
			method: Mock_Method2::new_mock(|mock| {
				mock.expect_compute_path()
					.returning(|_, _, _| Box::new([_Node(2, 2, 2)].into_iter()));
			}),
			graph: Mock_Graph::new_mock(|mock| {
				mock.expect_node()
					.with(eq(Vec3::new(0.8, 1., 1.3)))
					.return_const(Some(_Node(1, 1, 1)));
				mock.expect_node()
					.with(eq(Vec3::new(2.1, 2., 1.9)))
					.return_const(Some(_Node(2, 2, 2)));
				mock.expect_translation()
					.returning(|_Node(x, y, z)| Vec3::new(*x as f32, *y as f32, *z as f32));
				mock.expect_naive_path().never().return_const(NaivePath::Ok);
			}),
		};

		let path = grid.compute_path(
			Vec3::new(0.8, 1., 1.3),
			Vec3::new(2.1, 2., 1.9),
			Units::from(0.1),
		);
		assert_eq!(Some(vec![Vec3::new(2., 2., 2.)]), path);
	}
}
