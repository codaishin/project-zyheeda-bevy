mod clearance;
mod edge;
mod line;

#[cfg(debug_assertions)]
pub(crate) mod debug;

use crate::{
	mesh_grid_graph::clearance::{Clearance, SetClearance},
	systems::spawn_grid::TryFromTriangles,
	traits::to_subdivided::{SubdivisionError, ToSubdivided},
};
use bevy::{math::InvalidDirectionError, prelude::*};
use common::{
	tools::{Units, vec_not_nan::VecNotNan},
	traits::handles_map_generation::{
		Graph,
		GraphGroundPosition,
		GraphLineOfSight,
		GraphNaivePath,
		GraphNode,
		GraphObstacle,
		GraphSuccessors,
		GroundPosition,
		NaivePath,
	},
};
use core::f32;
use std::{
	cmp::Ordering,
	collections::{BinaryHeap, HashMap, HashSet, VecDeque},
	fmt::{Debug, Display},
	hash::Hash,
	ops::Deref,
};

#[derive(Debug, Clone, Default)]
#[cfg_attr(not(test), derive(PartialEq))]
pub struct MeshGridGraph {
	vertices: Vec<VecNotNan<3>>,
	neighbors: Vec<Vec<NodeId>>,
	clearance: Vec<Clearance>,
}

impl MeshGridGraph {
	fn new_node(&mut self, vec: VecNotNan<3>) -> NodeId {
		self.vertices.push(vec);
		self.neighbors.push(Vec::default());
		self.clearance.push(Clearance::INFINITY);

		NodeId(self.vertices.len() - 1)
	}

	fn insert_unique_neighbors(
		&mut self,
		node: NodeId,
		new_neighbors: impl IntoIterator<Item = NodeId>,
	) {
		let neighbors = &mut self.neighbors[*node];

		for neighbor in new_neighbors {
			if neighbor == node {
				continue;
			}

			if neighbors.contains(&neighbor) {
				continue;
			}

			neighbors.push(neighbor);
		}
	}
}

impl ToSubdivided for MeshGridGraph {
	fn to_subdivided(&self, _: u8) -> Result<Self, SubdivisionError> {
		Err(SubdivisionError::CannotSubdivide)
	}
}

impl TryFromTriangles for MeshGridGraph {
	type TError = TriangleEdgeError;

	fn try_from_triangles<TTriangles>(triangles: TTriangles) -> Result<Self, Self::TError>
	where
		TTriangles: Iterator<Item = [VecNotNan<3>; 3]>,
	{
		let mut ids = HashMap::<VecNotNan<3>, NodeId>::default();
		let mut nodes_facing_same_edge =
			HashMap::<edge::Edge, Vec<(NodeId, VecNotNan<3>)>>::default();
		let mut boundary_nodes = HashSet::<NodeId>::default();

		let mut graph = Self::default();

		for [a, b, c] in triangles {
			let id_a = *ids.entry(a).or_insert_with(|| graph.new_node(a));
			let id_b = *ids.entry(b).or_insert_with(|| graph.new_node(b));
			let id_c = *ids.entry(c).or_insert_with(|| graph.new_node(c));

			graph.insert_unique_neighbors(id_a, [id_b, id_c]);
			graph.insert_unique_neighbors(id_b, [id_a, id_c]);
			graph.insert_unique_neighbors(id_c, [id_a, id_b]);

			if id_a == id_b || id_a == id_c || id_b == id_c {
				continue;
			}

			nodes_facing_same_edge
				.entry(edge::Edge::uniform((id_b, b), (id_c, c)))
				.or_default()
				.push((id_a, a));
			nodes_facing_same_edge
				.entry(edge::Edge::uniform((id_a, a), (id_c, c)))
				.or_default()
				.push((id_b, b));
			nodes_facing_same_edge
				.entry(edge::Edge::uniform((id_a, a), (id_b, b)))
				.or_default()
				.push((id_c, c));
		}

		for (edge, nodes_facing_edge) in nodes_facing_same_edge {
			match nodes_facing_edge.as_slice() {
				[_, _, _, ..] => return Err(TriangleEdgeError::from(edge)),
				[] => {}
				[_] => {
					let (id0, id1) = edge.ids();
					boundary_nodes.insert(id0);
					boundary_nodes.insert(id1);
				}
				[(id_a, a), (id_b, b)] => {
					let a = Vec3::from(a);
					let b = Vec3::from(b);

					if !edge.crossed_by(a, b) {
						continue;
					}

					if !edge.spans_obtuse_angle_to(a) {
						continue;
					}

					if !edge.spans_obtuse_angle_to(b) {
						continue;
					}

					graph.insert_unique_neighbors(*id_a, *id_b);
					graph.insert_unique_neighbors(*id_b, *id_a);
				}
			};
		}

		let mut set_clearance = SetClearance::from_boundary(boundary_nodes);

		while let Some(step) = set_clearance.next_step() {
			step.process(&mut graph);
		}

		Ok(graph)
	}
}

impl Graph for MeshGridGraph {
	type TNode = NodeId;
}

impl GraphNode for MeshGridGraph {
	type TNNode = NodeId;

	fn node(&self, translation: Vec3) -> Option<Self::TNNode> {
		let mut closest_distance = f32::INFINITY;
		let mut closest = None;

		for (i, pos) in self.vertices.iter().enumerate() {
			if self.clearance[i] == Clearance::NONE {
				continue;
			}

			let distance = (translation - Vec3::from(*pos)).length_squared();
			if distance < closest_distance {
				closest_distance = distance;
				closest = Some(NodeId(i));
			}

			if closest_distance == 0. {
				break;
			}
		}

		closest
	}
}

impl GraphSuccessors for MeshGridGraph {
	type TSNode = NodeId;

	fn successors(&self, node: &Self::TSNode) -> impl Iterator<Item = Self::TSNode> {
		self.neighbors[**node].iter().copied()
	}
}

impl GraphLineOfSight for MeshGridGraph {
	type TLNode = NodeId;

	fn line_of_sight(&self, origin: &Self::TLNode, target: &Self::TLNode, width: Units) -> bool {
		let line = line::IterLine::new(*origin, *target, self, width);
		line.last() == Some(*target)
	}
}

impl GraphObstacle for MeshGridGraph {
	type TONode = NodeId;

	fn is_obstacle(&self, node: &Self::TONode, required_clearance: Units) -> bool {
		self.clearance[**node].is_zero_or_smaller_than(required_clearance)
	}
}

impl GraphGroundPosition for MeshGridGraph {
	type TTNode = NodeId;

	fn ground_position(&self, node: &Self::TTNode) -> GroundPosition {
		GroundPosition(Vec3::from(self.vertices[**node]))
	}
}

impl GraphNaivePath for MeshGridGraph {
	type TNNode = NodeId;

	fn naive_path(
		&self,
		translation: Vec3,
		target: &Self::TNNode,
		required_clearance: Units,
	) -> NaivePath {
		let line = self
			.node(translation)
			.map(|start| line::IterLine::new(start, *target, self, required_clearance))
			.into_iter()
			.flatten();

		match line.last() {
			Some(last) if &last == target => NaivePath::Ok,
			Some(last) => NaivePath::PartialUntil(self.ground_position(&last)),
			None => NaivePath::CannotCompute,
		}
	}
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct NodeId(usize);

impl Deref for NodeId {
	type Target = usize;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl IntoIterator for NodeId {
	type Item = NodeId;
	type IntoIter = std::iter::Once<NodeId>;

	fn into_iter(self) -> Self::IntoIter {
		std::iter::once(self)
	}
}

#[derive(Debug, PartialEq)]
pub(crate) struct TriangleEdgeError(VecNotNan<3>, VecNotNan<3>);

impl Display for TriangleEdgeError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"Edge <{}, {}> shared by more than 2 triangles",
			Vec3::from(self.0),
			Vec3::from(self.1),
		)
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use common::vec3_not_nan;

	macro_rules! neighbors {
		(node($($n:expr),* $(,)?)) => {
			Vec::from([$($n),*].map(NodeId))
		};
		($([$($n:expr),* $(,)?]),* $(,)?) => {
			vec![
				$(neighbors!(node($($n),*))),*
			]
		};
	}

	pub(crate) use neighbors;

	impl PartialEq for MeshGridGraph {
		fn eq(&self, other: &Self) -> bool {
			if self.vertices != other.vertices {
				return false;
			}

			if self.clearance != other.clearance {
				return false;
			}

			if self.neighbors.len() != other.neighbors.len() {
				return false;
			}

			for (s, o) in self.neighbors.iter().zip(&other.neighbors) {
				if s.len() != o.len() {
					return false;
				}

				if s.iter().collect::<HashSet<_>>() != o.iter().collect::<HashSet<_>>() {
					return false;
				}
			}

			true
		}
	}

	mod instantiation {
		use super::*;
		use test_case::test_case;

		/// ```
		///  a — b
		///    \ |
		///      c
		/// ```
		#[test]
		fn set_nodes_of_one_triangle() {
			let a = vec3_not_nan!(0., 0., 0.);
			let b = vec3_not_nan!(1., 0., 0.);
			let c = vec3_not_nan!(1., 0., 1.);

			let graph = MeshGridGraph::try_from_triangles([[a, b, c]].into_iter());

			assert_eq!(
				Ok(MeshGridGraph {
					vertices: vec![a, b, c],
					neighbors: neighbors![[1, 2], [0, 2], [0, 1],],
					clearance: vec![Clearance::NONE; 3],
				}),
				graph
			);
		}

		/// ```
		///  a — b
		///  | \ |
		///  d — c
		/// ```
		#[test]
		fn set_nodes_of_two_connected_triangles() {
			let a = vec3_not_nan!(0., 0., 0.);
			let b = vec3_not_nan!(1., 0., 0.);
			let c = vec3_not_nan!(1., 0., 1.);
			let d = vec3_not_nan!(0., 0., 1.);

			let graph = MeshGridGraph::try_from_triangles([[a, b, c], [a, c, d]].into_iter());

			assert_eq!(
				Ok(MeshGridGraph {
					vertices: vec![a, b, c, d],
					neighbors: neighbors![[1, 2, 3], [0, 2, 3], [0, 1, 3], [0, 1, 2]],
					clearance: vec![Clearance::NONE; 4],
				}),
				graph,
			);
		}

		/// ```
		///  a — b
		///  | \ |
		///  d — c
		/// ```
		#[test]
		fn set_nodes_of_two_connected_triangles_when_edge_reversed() {
			let a = vec3_not_nan!(0., 0., 0.);
			let b = vec3_not_nan!(1., 0., 0.);
			let c = vec3_not_nan!(1., 0., 1.);
			let d = vec3_not_nan!(0., 0., 1.);

			let graph = MeshGridGraph::try_from_triangles([[a, b, c], [c, a, d]].into_iter());

			assert_eq!(
				Ok(MeshGridGraph {
					vertices: vec![a, b, c, d],
					neighbors: neighbors![[1, 2, 3], [0, 2, 3], [0, 1, 3], [0, 1, 2]],
					clearance: vec![Clearance::NONE; 4],
				}),
				graph,
			);
		}

		/// ```
		///  a — b
		///    \ |
		///      b
		/// ```
		#[test]
		fn prevent_node_neighbor_self_reference() {
			let a = vec3_not_nan!(0., 0., 0.);
			let b = vec3_not_nan!(1., 0., 0.);

			let graph = MeshGridGraph::try_from_triangles([[a, b, b]].into_iter());

			assert_eq!(
				Ok(MeshGridGraph {
					vertices: vec![a, b],
					neighbors: neighbors![[1], [0]],
					clearance: vec![Clearance::INFINITY; 2],
				}),
				graph,
			);
		}

		/// ```
		/// a         d
		///  \ ＼  ／ /
		///   \  b  /
		///    \ | /
		///      c
		/// ```
		#[test]
		fn do_not_connect_across_shared_edge_when_concave() {
			let a = vec3_not_nan!(0., 0., 0.);
			let b = vec3_not_nan!(1., 0., 0.1);
			let c = vec3_not_nan!(1., 0., 1.1);
			let d = vec3_not_nan!(2., 0., 0.);

			let graph = MeshGridGraph::try_from_triangles([[a, b, c], [b, c, d]].into_iter());

			assert_eq!(
				Ok(MeshGridGraph {
					vertices: vec![a, b, c, d],
					neighbors: neighbors![[1, 2], [0, 2, 3], [0, 1, 3], [1, 2]],
					clearance: vec![Clearance::NONE; 4],
				}),
				graph,
			);
		}

		/// ```
		///      b
		///    / | \
		///   /  c  \
		///  / ／  ＼ \
		/// a         d
		/// ```
		#[test]
		fn do_not_connect_across_shared_edge_when_concave_reversed() {
			let a = vec3_not_nan!(0., 0., 1.1);
			let b = vec3_not_nan!(1., 0., 0.);
			let c = vec3_not_nan!(1., 0., 1.);
			let d = vec3_not_nan!(2., 0., 1.1);

			let graph = MeshGridGraph::try_from_triangles([[a, b, c], [b, c, d]].into_iter());

			assert_eq!(
				Ok(MeshGridGraph {
					vertices: vec![a, b, c, d],
					neighbors: neighbors![[1, 2], [0, 2, 3], [0, 1, 3], [1, 2]],
					clearance: vec![Clearance::NONE; 4],
				}),
				graph,
			);
		}

		/// Triangles
		/// ```
		///  a — b — e
		///  | \ | \ |
		///  d — c — f
		/// ```
		///
		/// Connect:
		/// - b-d
		/// - e-c
		///
		/// Do not connect:
		/// - a-f
		///
		/// Expected result
		/// ```
		///  a — b — e
		///  | X | X |
		///  d — c — f
		/// ```
		#[test]
		fn do_not_connect_nodes_on_acute_angles() {
			let a = vec3_not_nan!(0., 0., 0.);
			let b = vec3_not_nan!(1., 0., 0.);
			let c = vec3_not_nan!(1., 0., 1.);
			let d = vec3_not_nan!(0., 0., 1.);
			let e = vec3_not_nan!(2., 0., 0.);
			let f = vec3_not_nan!(2., 0., 1.);

			let graph = MeshGridGraph::try_from_triangles(
				[[a, b, c], [a, d, c], [b, e, f], [b, c, f]].into_iter(),
			);

			assert_eq!(
				Ok(MeshGridGraph {
					vertices: vec![a, b, c, d, e, f],
					neighbors: neighbors![
						[1, 2, 3],
						[0, 2, 3, 4, 5],
						[0, 1, 3, 4, 5],
						[0, 1, 2],
						[1, 2, 5],
						[1, 2, 4],
					],
					clearance: vec![Clearance::NONE; 6],
				}),
				graph,
			);
		}

		/// ```
		///  a — b
		///  | \ |
		///  d — c
		/// ```
		#[test]
		fn set_nodes_of_two_connected_triangles_within_minimal_error() {
			let a = vec3_not_nan!(0., 0., 0.);
			let b = vec3_not_nan!(1., 0., -1e-6);
			let c = vec3_not_nan!(1., 0., 1.);
			let d = vec3_not_nan!(0., 0., 1.);

			let graph = MeshGridGraph::try_from_triangles([[a, b, c], [a, c, d]].into_iter());

			assert_eq!(
				Ok(MeshGridGraph {
					vertices: vec![a, b, c, d],
					neighbors: neighbors![[1, 2, 3], [0, 2, 3], [0, 1, 3], [0, 1, 2]],
					clearance: vec![Clearance::NONE; 4],
				}),
				graph,
			);
		}

		/// ```
		///      b
		///    / | \
		///  b — i — b
		///    \ | /
		///      b
		/// ```
		#[test]
		fn identify_clearance() {
			let inner = vec3_not_nan!(0., 0., 0.);
			let boundary_vertices = [
				vec3_not_nan!(-1., 0., 0.),
				vec3_not_nan!(0., 0., -1.),
				vec3_not_nan!(1., 0., 0.),
				vec3_not_nan!(0., 0., 1.),
			];
			let triangles = [
				[inner, boundary_vertices[0], boundary_vertices[1]],
				[inner, boundary_vertices[1], boundary_vertices[2]],
				[inner, boundary_vertices[2], boundary_vertices[3]],
				[inner, boundary_vertices[3], boundary_vertices[0]],
			];

			let graph = MeshGridGraph::try_from_triangles(triangles.into_iter());

			assert_eq!(
				Ok(MeshGridGraph {
					vertices: vec![
						inner,
						boundary_vertices[0],
						boundary_vertices[1],
						boundary_vertices[2],
						boundary_vertices[3],
					],
					neighbors: neighbors![[1, 2, 3, 4], [0, 2, 4], [0, 1, 3], [0, 2, 4], [0, 1, 3]],
					clearance: vec![
						Clearance::from(Units::from(1.0)),
						Clearance::NONE,
						Clearance::NONE,
						Clearance::NONE,
						Clearance::NONE,
					],
				}),
				graph,
			);
		}

		/// ```
		///      A   B   C
		///    b — b — b — b
		/// 0  | / | / | / |
		///    b — i — i — b
		/// 1  | / | / | / |
		///    b — b — b — b
		/// 2  | / |   | / |
		///    b — b   b — b
		/// ```
		#[test]
		fn identify_clearance_complex_1() {
			let triangles = [
				// A0
				[
					vec3_not_nan!(0., 0., 0.),
					vec3_not_nan!(0., 0., 1.),
					vec3_not_nan!(1., 0., 0.),
				],
				[
					vec3_not_nan!(0., 0., 1.),
					vec3_not_nan!(1., 0., 0.),
					vec3_not_nan!(1., 0., 1.),
				],
				// B0
				[
					vec3_not_nan!(1., 0., 0.),
					vec3_not_nan!(1., 0., 1.),
					vec3_not_nan!(2., 0., 0.),
				],
				[
					vec3_not_nan!(1., 0., 1.),
					vec3_not_nan!(2., 0., 0.),
					vec3_not_nan!(2., 0., 1.),
				],
				// C0
				[
					vec3_not_nan!(2., 0., 0.),
					vec3_not_nan!(2., 0., 1.),
					vec3_not_nan!(3., 0., 0.),
				],
				[
					vec3_not_nan!(2., 0., 1.),
					vec3_not_nan!(3., 0., 0.),
					vec3_not_nan!(3., 0., 1.),
				],
				// A1
				[
					vec3_not_nan!(0., 0., 1.),
					vec3_not_nan!(0., 0., 2.),
					vec3_not_nan!(1., 0., 1.),
				],
				[
					vec3_not_nan!(0., 0., 2.),
					vec3_not_nan!(1., 0., 1.),
					vec3_not_nan!(1., 0., 2.),
				],
				// B1
				[
					vec3_not_nan!(1., 0., 1.),
					vec3_not_nan!(1., 0., 2.),
					vec3_not_nan!(2., 0., 1.),
				],
				[
					vec3_not_nan!(1., 0., 2.),
					vec3_not_nan!(2., 0., 1.),
					vec3_not_nan!(2., 0., 2.),
				],
				// C1
				[
					vec3_not_nan!(2., 0., 1.),
					vec3_not_nan!(2., 0., 2.),
					vec3_not_nan!(3., 0., 1.),
				],
				[
					vec3_not_nan!(2., 0., 2.),
					vec3_not_nan!(3., 0., 1.),
					vec3_not_nan!(3., 0., 2.),
				],
				// A2
				[
					vec3_not_nan!(0., 0., 2.),
					vec3_not_nan!(0., 0., 3.),
					vec3_not_nan!(1., 0., 2.),
				],
				[
					vec3_not_nan!(0., 0., 3.),
					vec3_not_nan!(1., 0., 2.),
					vec3_not_nan!(1., 0., 3.),
				],
				// C2
				[
					vec3_not_nan!(2., 0., 2.),
					vec3_not_nan!(3., 0., 2.),
					vec3_not_nan!(2., 0., 3.),
				],
				[
					vec3_not_nan!(3., 0., 2.),
					vec3_not_nan!(2., 0., 3.),
					vec3_not_nan!(3., 0., 3.),
				],
			];

			let graph = MeshGridGraph::try_from_triangles(triangles.into_iter());

			assert_eq!(
				Ok(vec![
					// Line 0
					Clearance::NONE,
					Clearance::NONE,
					Clearance::NONE,
					Clearance::from(Units::from(1.0)),
					Clearance::NONE,
					Clearance::from(Units::from(1.0)),
					Clearance::NONE,
					Clearance::NONE,
					// Line 1 remainder
					Clearance::NONE,
					Clearance::NONE,
					Clearance::NONE,
					Clearance::NONE,
					// Line 1 remainder
					Clearance::NONE,
					Clearance::NONE,
					Clearance::NONE,
					Clearance::NONE,
				]),
				graph.map(|g| g.clearance),
			);
		}

		/// ```
		///      A   B   C   D
		///    b — b — b — b — b
		/// 0  | / | / | / | / |
		///    b — i — i — i — b
		/// 1  | / | / | / | / |
		///    b — i — i — i — b
		/// 2  | / | / | / | / |
		///    b — i — i — i — b
		/// 3  | / | / | / | / |
		///    b — b — b — b — b
		/// ```
		#[test]
		fn identify_clearance_complex_2() {
			let triangles = [
				// A0
				[
					vec3_not_nan!(0., 0., 0.),
					vec3_not_nan!(0., 0., 1.),
					vec3_not_nan!(1., 0., 0.),
				],
				[
					vec3_not_nan!(0., 0., 1.),
					vec3_not_nan!(1., 0., 0.),
					vec3_not_nan!(1., 0., 1.),
				],
				// B0
				[
					vec3_not_nan!(1., 0., 0.),
					vec3_not_nan!(1., 0., 1.),
					vec3_not_nan!(2., 0., 0.),
				],
				[
					vec3_not_nan!(1., 0., 1.),
					vec3_not_nan!(2., 0., 0.),
					vec3_not_nan!(2., 0., 1.),
				],
				// C0
				[
					vec3_not_nan!(2., 0., 0.),
					vec3_not_nan!(2., 0., 1.),
					vec3_not_nan!(3., 0., 0.),
				],
				[
					vec3_not_nan!(2., 0., 1.),
					vec3_not_nan!(3., 0., 0.),
					vec3_not_nan!(3., 0., 1.),
				],
				// D0
				[
					vec3_not_nan!(3., 0., 0.),
					vec3_not_nan!(3., 0., 1.),
					vec3_not_nan!(4., 0., 0.),
				],
				[
					vec3_not_nan!(3., 0., 1.),
					vec3_not_nan!(4., 0., 0.),
					vec3_not_nan!(4., 0., 1.),
				],
				// A1
				[
					vec3_not_nan!(0., 0., 1.),
					vec3_not_nan!(0., 0., 2.),
					vec3_not_nan!(1., 0., 1.),
				],
				[
					vec3_not_nan!(0., 0., 2.),
					vec3_not_nan!(1., 0., 1.),
					vec3_not_nan!(1., 0., 2.),
				],
				// B1
				[
					vec3_not_nan!(1., 0., 1.),
					vec3_not_nan!(1., 0., 2.),
					vec3_not_nan!(2., 0., 1.),
				],
				[
					vec3_not_nan!(1., 0., 2.),
					vec3_not_nan!(2., 0., 1.),
					vec3_not_nan!(2., 0., 2.),
				],
				// C1
				[
					vec3_not_nan!(2., 0., 1.),
					vec3_not_nan!(2., 0., 2.),
					vec3_not_nan!(3., 0., 1.),
				],
				[
					vec3_not_nan!(2., 0., 2.),
					vec3_not_nan!(3., 0., 1.),
					vec3_not_nan!(3., 0., 2.),
				],
				// D1
				[
					vec3_not_nan!(3., 0., 1.),
					vec3_not_nan!(3., 0., 2.),
					vec3_not_nan!(4., 0., 1.),
				],
				[
					vec3_not_nan!(3., 0., 2.),
					vec3_not_nan!(4., 0., 1.),
					vec3_not_nan!(4., 0., 2.),
				],
				// A2
				[
					vec3_not_nan!(0., 0., 2.),
					vec3_not_nan!(0., 0., 3.),
					vec3_not_nan!(1., 0., 2.),
				],
				[
					vec3_not_nan!(0., 0., 3.),
					vec3_not_nan!(1., 0., 2.),
					vec3_not_nan!(1., 0., 3.),
				],
				// B2
				[
					vec3_not_nan!(1., 0., 2.),
					vec3_not_nan!(1., 0., 3.),
					vec3_not_nan!(2., 0., 2.),
				],
				[
					vec3_not_nan!(1., 0., 3.),
					vec3_not_nan!(2., 0., 2.),
					vec3_not_nan!(2., 0., 3.),
				],
				// C2
				[
					vec3_not_nan!(2., 0., 2.),
					vec3_not_nan!(2., 0., 3.),
					vec3_not_nan!(3., 0., 2.),
				],
				[
					vec3_not_nan!(2., 0., 3.),
					vec3_not_nan!(3., 0., 2.),
					vec3_not_nan!(3., 0., 3.),
				],
				// D2
				[
					vec3_not_nan!(3., 0., 2.),
					vec3_not_nan!(3., 0., 3.),
					vec3_not_nan!(4., 0., 2.),
				],
				[
					vec3_not_nan!(3., 0., 3.),
					vec3_not_nan!(4., 0., 2.),
					vec3_not_nan!(4., 0., 3.),
				],
				// A3
				[
					vec3_not_nan!(0., 0., 3.),
					vec3_not_nan!(0., 0., 4.),
					vec3_not_nan!(1., 0., 3.),
				],
				[
					vec3_not_nan!(0., 0., 4.),
					vec3_not_nan!(1., 0., 3.),
					vec3_not_nan!(1., 0., 4.),
				],
				// B3
				[
					vec3_not_nan!(1., 0., 3.),
					vec3_not_nan!(1., 0., 4.),
					vec3_not_nan!(2., 0., 3.),
				],
				[
					vec3_not_nan!(1., 0., 4.),
					vec3_not_nan!(2., 0., 3.),
					vec3_not_nan!(2., 0., 4.),
				],
				// C3
				[
					vec3_not_nan!(2., 0., 3.),
					vec3_not_nan!(2., 0., 4.),
					vec3_not_nan!(3., 0., 3.),
				],
				[
					vec3_not_nan!(2., 0., 4.),
					vec3_not_nan!(3., 0., 3.),
					vec3_not_nan!(3., 0., 4.),
				],
				// D3
				[
					vec3_not_nan!(3., 0., 3.),
					vec3_not_nan!(3., 0., 4.),
					vec3_not_nan!(4., 0., 3.),
				],
				[
					vec3_not_nan!(3., 0., 4.),
					vec3_not_nan!(4., 0., 3.),
					vec3_not_nan!(4., 0., 4.),
				],
			];

			let graph = MeshGridGraph::try_from_triangles(triangles.into_iter());

			assert_eq!(
				Ok(vec![
					// Line 0
					Clearance::NONE,
					Clearance::NONE,
					Clearance::NONE,
					Clearance::from(Units::from(1.0)),
					Clearance::NONE,
					Clearance::from(Units::from(1.0)),
					Clearance::NONE,
					Clearance::from(Units::from(1.0)),
					Clearance::NONE,
					Clearance::NONE,
					// Line 1 remainder
					Clearance::NONE,
					Clearance::from(Units::from(1.0)),
					Clearance::from(Units::from(2.0)),
					Clearance::from(Units::from(1.0)),
					Clearance::NONE,
					// Line 2 remainder
					Clearance::NONE,
					Clearance::from(Units::from(1.0)),
					Clearance::from(Units::from(1.0)),
					Clearance::from(Units::from(1.0)),
					Clearance::NONE,
					// Line 3 remainder
					Clearance::NONE,
					Clearance::NONE,
					Clearance::NONE,
					Clearance::NONE,
					Clearance::NONE,
				]),
				graph.map(|g| g.clearance),
			);
		}

		#[test_case([vec3_not_nan!(1., 2., 5.), vec3_not_nan!(1., 2., 6.), vec3_not_nan!(1., 2., 7.)]; "3")]
		#[test_case([vec3_not_nan!(1., 2., 5.), vec3_not_nan!(1., 2., 6.), vec3_not_nan!(1., 2., 7.), vec3_not_nan!(1., 2., 8.)]; "4")]
		fn return_error_when_more_than_2_triangles_share_same_edge<const N: usize>(
			other_vertices: [VecNotNan<3>; N],
		) {
			let edge = (vec3_not_nan!(1., 2., 3.), vec3_not_nan!(1., 2., 4.));
			let triangles = other_vertices.map(|o| [edge.0, edge.1, o]);

			let graph = MeshGridGraph::try_from_triangles(triangles.into_iter());

			assert_eq!(
				Err(TriangleEdgeError(
					vec3_not_nan!(1., 2., 3.),
					vec3_not_nan!(1., 2., 4.)
				)),
				graph,
			);
		}
	}

	mod graph_node {
		use super::*;

		#[test]
		fn get_exact_translation() {
			let graph = MeshGridGraph {
				vertices: vec![vec3_not_nan!(1., 2., 3.), vec3_not_nan!(4., 5., 6.)],
				neighbors: neighbors![[], []],
				clearance: vec![Clearance::INFINITY; 2],
			};

			let node = graph.node(Vec3::new(4., 5., 6.));

			assert_eq!(Some(NodeId(1)), node);
		}

		#[test]
		fn get_closest_translation() {
			let graph = MeshGridGraph {
				vertices: vec![vec3_not_nan!(10., 2., 3.), vec3_not_nan!(1., 2., 3.)],
				neighbors: neighbors![[], []],
				clearance: vec![Clearance::INFINITY; 2],
			};

			let node = graph.node(Vec3::new(2., 3., 4.));

			assert_eq!(Some(NodeId(1)), node);
		}

		#[test]
		fn get_closest_translation_with_non_zero_clearance() {
			let graph = MeshGridGraph {
				vertices: vec![
					vec3_not_nan!(10., 2., 3.),
					vec3_not_nan!(1., 2., 3.),
					vec3_not_nan!(1., 2., 2.),
				],
				neighbors: neighbors![[], [], []],
				clearance: vec![Clearance::INFINITY, Clearance::NONE, Clearance::INFINITY],
			};

			let node = graph.node(Vec3::new(2., 3., 4.));

			assert_eq!(Some(NodeId(2)), node);
		}
	}

	mod successors {
		use super::*;

		#[test]
		fn neighbors_as_successors() {
			let graph = MeshGridGraph {
				vertices: vec![vec3_not_nan!(1., 2., 3.)],
				neighbors: neighbors![[1, 2]],
				clearance: vec![Clearance::INFINITY; 2],
			};

			let successors = graph.successors(&NodeId(0));

			assert_eq!(
				HashSet::from([NodeId(1), NodeId(2)]),
				successors.collect::<HashSet<_>>(),
			);
		}
	}
}
