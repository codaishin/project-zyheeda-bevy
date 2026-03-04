use crate::{
	mesh_grid_graph::edge::Edge,
	systems::spawn_grid::TryFromTriangles,
	traits::to_subdivided::{SubdivisionError, ToSubdivided},
};
use bevy::{math::InvalidDirectionError, prelude::*};
use common::{
	tools::vec_not_nan::Vec3NotNan,
	traits::handles_map_generation::{GraphLineOfSight, GraphNode, GraphSuccessors},
};
use core::f32;
use std::{
	cmp::Ordering,
	collections::{HashMap, HashSet, VecDeque},
	fmt::{Debug, Display},
	hash::Hash,
};

#[derive(Debug, PartialEq, Clone, Default)]
pub struct MeshGridGraph(HashMap<Vec3NotNan, HashSet<Vec3NotNan>>);

impl ToSubdivided for MeshGridGraph {
	fn to_subdivided(&self, _: u8) -> Result<Self, SubdivisionError> {
		Err(SubdivisionError::CannotSubdivide)
	}
}

impl TryFromTriangles for MeshGridGraph {
	type TError = TriangleEdgeError;

	fn try_from_triangles<TTriangles>(triangles: TTriangles) -> Result<Self, Self::TError>
	where
		TTriangles: Iterator<Item = [Vec3NotNan; 3]>,
	{
		let mut graph = HashMap::<Vec3NotNan, HashSet<Vec3NotNan>>::default();
		let mut nodes_facing_same_edge = HashMap::<Edge, Vec<Vec3NotNan>>::default();

		for [a, b, c] in triangles {
			graph.entry(a).or_default().extend([b, c]);
			nodes_facing_same_edge
				.entry(Edge::uniform(b, c))
				.or_default()
				.push(a);

			graph.entry(b).or_default().extend([a, c]);
			nodes_facing_same_edge
				.entry(Edge::uniform(a, c))
				.or_default()
				.push(b);

			graph.entry(c).or_default().extend([a, b]);
			nodes_facing_same_edge
				.entry(Edge::uniform(a, b))
				.or_default()
				.push(c);
		}

		for (edge, nodes) in nodes_facing_same_edge {
			let (a, b) = match nodes.as_slice() {
				[] | [_] => continue,
				[a, b] => (*a, *b),
				[_, _, _, ..] => return Err(TriangleEdgeError::from(edge)),
			};

			graph.entry(a).or_default().insert(b);
			graph.entry(b).or_default().insert(a);
		}

		Ok(Self(graph))
	}
}

impl GraphNode for MeshGridGraph {
	type TNNode = Vec3NotNan;

	fn node(&self, translation: Vec3) -> Option<Self::TNNode> {
		struct Closest<'a> {
			node: &'a Vec3NotNan,
			distance: f32,
		}

		let mut nodes = self.0.keys();
		let node = nodes.next()?;
		let mut closest = Closest {
			node,
			distance: (translation - **node).length(),
		};

		for node in nodes {
			// untested short circuit
			if closest.distance == 0. {
				return Some(*closest.node);
			}

			let distance = (translation - **node).length();
			if distance > closest.distance {
				continue;
			}

			closest = Closest { node, distance };
		}

		Some(*closest.node)
	}
}

impl GraphSuccessors for MeshGridGraph {
	type TSNode = Vec3NotNan;

	fn successors(&self, node: &Self::TSNode) -> impl Iterator<Item = Self::TSNode> {
		self.0.get(node).into_iter().flat_map(|n| n.iter().copied())
	}
}

impl GraphLineOfSight for MeshGridGraph {
	type TLNode = Vec3NotNan;

	fn line_of_sight(&self, origin: &Self::TLNode, target: &Self::TLNode) -> bool {
		let Some(target_neighbors) = self.0.get(target) else {
			return false;
		};

		let mut seen = HashSet::from([]);
		let mut open = VecDeque::from([*origin]);

		while let Some(current) = open.pop_front() {
			let Some(current_neighbors) = self.0.get(&current) else {
				continue;
			};

			let direction = match Dir3::try_from(**target - *current) {
				Err(InvalidDirectionError::Zero) => return true,
				Err(_) => continue,
				Ok(direction) => direction,
			};

			if target_neighbors.contains(&current) {
				return true;
			}

			let forward_edge = los::forward_edge(
				&current,
				direction,
				current_neighbors.iter(),
				move |a, b| self.0.get(&a).map(|n| n.contains(&b)).unwrap_or(false),
			);

			for corner in forward_edge {
				if target_neighbors.contains(&corner) {
					return true;
				}

				if !seen.contains(&corner) {
					seen.insert(corner);
					open.push_back(corner);
				}
			}
		}

		false
	}
}

#[derive(Debug, PartialEq)]
pub(crate) struct TriangleEdgeError(Vec3NotNan, Vec3NotNan);

impl Display for TriangleEdgeError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"Edge <{}, {}> shared by more than 2 triangles",
			*self.0, *self.1
		)
	}
}

mod edge {
	use super::*;
	use std::ops::Deref;

	#[derive(Debug, PartialEq, Eq, Hash)]
	pub(super) struct Edge((Vec3NotNan, Vec3NotNan));

	impl Edge {
		pub(super) fn uniform(a: Vec3NotNan, b: Vec3NotNan) -> Self {
			match a.cmp(&b) {
				Ordering::Less => Self((a, b)),
				_ => Self((b, a)),
			}
		}
	}

	impl From<Edge> for TriangleEdgeError {
		fn from(Edge((a, b)): Edge) -> Self {
			Self(a, b)
		}
	}

	impl Deref for Edge {
		type Target = (Vec3NotNan, Vec3NotNan);

		fn deref(&self) -> &Self::Target {
			&self.0
		}
	}
}

mod los {
	use super::*;

	pub(super) enum IterateNodes {
		None,
		One(Vec3NotNan),
		Two(Vec3NotNan, Vec3NotNan),
	}

	impl Iterator for IterateNodes {
		type Item = Vec3NotNan;

		fn next(&mut self) -> Option<Self::Item> {
			match *self {
				Self::None => None,
				Self::One(one) => {
					*self = Self::None;
					Some(one)
				}
				Self::Two(fst, snd) => {
					*self = Self::One(snd);
					Some(fst)
				}
			}
		}
	}

	enum Forward {
		Collinear,
		Left(f32),
		Right(f32),
	}

	const MIN_ORIENTATION_DELTA: f32 = 1e-5;

	fn orientation(a: Dir3, b: Dir3) -> Option<Forward> {
		if a.dot(*b) < 0. {
			return None;
		}

		let forward = match a.cross(*b).y {
			c if c.abs() < MIN_ORIENTATION_DELTA => Forward::Collinear,
			c if c > 0. => Forward::Left(c.abs()),
			c => Forward::Right(c.abs()),
		};

		Some(forward)
	}

	pub(super) fn forward_edge<'a>(
		node: &Vec3NotNan,
		dir: Dir3,
		neighbors: impl Iterator<Item = &'a Vec3NotNan>,
		is_edge: impl Fn(Vec3NotNan, Vec3NotNan) -> bool,
	) -> IterateNodes {
		let mut left = None;
		let mut collinear = None;
		let mut right = None;

		for n in neighbors {
			let Ok(n_dir) = Dir3::try_from(**n - **node) else {
				continue;
			};

			match orientation(n_dir, dir) {
				None => continue,
				Some(Forward::Collinear) => {
					// Track a single neighbor directly along `dir`.
					// If multiple collinear neighbors exist, the local topology
					// is inconsistent, so we only pick one. LOS may fail in that case,
					// but that is acceptable.
					collinear = Some(*n);
				}
				Some(Forward::Left(l_angle)) => {
					if left.map(|(_, angle)| l_angle < angle).unwrap_or(true) {
						left = Some((*n, l_angle));
					}
				}
				Some(Forward::Right(r_angle)) => {
					if right.map(|(_, angle)| r_angle < angle).unwrap_or(true) {
						right = Some((*n, r_angle));
					}
				}
			}
		}

		match (left, collinear, right) {
			(Some((l, l_angle)), None, Some((r, r_angle))) if is_edge(l, r) => {
				match l_angle.partial_cmp(&r_angle) {
					Some(Ordering::Equal) => IterateNodes::Two(l, r),
					Some(Ordering::Less) => IterateNodes::One(r),
					Some(Ordering::Greater) => IterateNodes::One(l),
					None => IterateNodes::None,
				}
			}
			(Some((l, ..)), Some(c), Some((r, ..))) if is_edge(l, c) && is_edge(r, c) => {
				IterateNodes::Two(l, r)
			}
			(Some((l, ..)), Some(c), ..) if is_edge(l, c) => IterateNodes::One(l),
			(.., Some(c), Some((r, ..))) if is_edge(r, c) => IterateNodes::One(r),
			_ => IterateNodes::None,
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use common::vec3_not_nan;

	mod instantiation {
		use super::*;
		use test_case::test_case;

		#[test]
		fn set_nodes_of_one_triangle() {
			let triangles = [[
				vec3_not_nan!(1., 2., 3.),
				vec3_not_nan!(1., 2., 4.),
				vec3_not_nan!(1., 2., 5.),
			]];

			let graph = MeshGridGraph::try_from_triangles(triangles.into_iter());

			assert_eq!(
				Ok(MeshGridGraph(HashMap::from([
					(
						vec3_not_nan!(1., 2., 3.),
						HashSet::from([vec3_not_nan!(1., 2., 4.), vec3_not_nan!(1., 2., 5.)])
					),
					(
						vec3_not_nan!(1., 2., 4.),
						HashSet::from([vec3_not_nan!(1., 2., 3.), vec3_not_nan!(1., 2., 5.)])
					),
					(
						vec3_not_nan!(1., 2., 5.),
						HashSet::from([vec3_not_nan!(1., 2., 4.), vec3_not_nan!(1., 2., 3.)])
					),
				]))),
				graph
			);
		}

		#[test]
		fn set_nodes_of_two_connected_triangles() {
			let triangles = [
				[
					vec3_not_nan!(1., 2., 3.),
					vec3_not_nan!(1., 2., 4.),
					vec3_not_nan!(1., 2., 5.),
				],
				[
					vec3_not_nan!(1., 2., 3.),
					vec3_not_nan!(1., 2., 4.),
					vec3_not_nan!(1., 2., 6.),
				],
			];

			let graph = MeshGridGraph::try_from_triangles(triangles.into_iter());

			assert_eq!(
				Ok(MeshGridGraph(HashMap::from([
					(
						vec3_not_nan!(1., 2., 3.),
						HashSet::from([
							vec3_not_nan!(1., 2., 4.),
							vec3_not_nan!(1., 2., 5.),
							vec3_not_nan!(1., 2., 6.),
						])
					),
					(
						vec3_not_nan!(1., 2., 4.),
						HashSet::from([
							vec3_not_nan!(1., 2., 3.),
							vec3_not_nan!(1., 2., 5.),
							vec3_not_nan!(1., 2., 6.),
						])
					),
					(
						vec3_not_nan!(1., 2., 5.),
						HashSet::from([
							vec3_not_nan!(1., 2., 4.),
							vec3_not_nan!(1., 2., 3.),
							vec3_not_nan!(1., 2., 6.),
						])
					),
					(
						vec3_not_nan!(1., 2., 6.),
						HashSet::from([
							vec3_not_nan!(1., 2., 4.),
							vec3_not_nan!(1., 2., 3.),
							vec3_not_nan!(1., 2., 5.),
						])
					),
				]))),
				graph
			);
		}

		#[test]
		fn connect_diagonal_of_square_when_shared_edge_is_reversed() {
			let triangles = [
				[
					vec3_not_nan!(1., 2., 3.),
					vec3_not_nan!(1., 2., 4.),
					vec3_not_nan!(1., 2., 5.),
				],
				[
					vec3_not_nan!(1., 2., 4.),
					vec3_not_nan!(1., 2., 3.), // reversed edge
					vec3_not_nan!(1., 2., 6.),
				],
			];

			let graph = MeshGridGraph::try_from_triangles(triangles.into_iter());

			assert_eq!(
				Ok(MeshGridGraph(HashMap::from([
					(
						vec3_not_nan!(1., 2., 3.),
						HashSet::from([
							vec3_not_nan!(1., 2., 4.),
							vec3_not_nan!(1., 2., 5.),
							vec3_not_nan!(1., 2., 6.),
						])
					),
					(
						vec3_not_nan!(1., 2., 4.),
						HashSet::from([
							vec3_not_nan!(1., 2., 3.),
							vec3_not_nan!(1., 2., 5.),
							vec3_not_nan!(1., 2., 6.),
						])
					),
					(
						vec3_not_nan!(1., 2., 5.),
						HashSet::from([
							vec3_not_nan!(1., 2., 3.),
							vec3_not_nan!(1., 2., 4.),
							vec3_not_nan!(1., 2., 6.),
						])
					),
					(
						vec3_not_nan!(1., 2., 6.),
						HashSet::from([
							vec3_not_nan!(1., 2., 3.),
							vec3_not_nan!(1., 2., 4.),
							vec3_not_nan!(1., 2., 5.),
						])
					),
				]))),
				graph
			);
		}

		#[test_case([vec3_not_nan!(1., 2., 5.), vec3_not_nan!(1., 2., 6.), vec3_not_nan!(1., 2., 7.)]; "3")]
		#[test_case([vec3_not_nan!(1., 2., 5.), vec3_not_nan!(1., 2., 6.), vec3_not_nan!(1., 2., 7.), vec3_not_nan!(1., 2., 8.)]; "4")]
		fn return_error_when_more_than_2_triangles_share_same_edge<const N: usize>(
			other_vertices: [Vec3NotNan; N],
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
			let graph = MeshGridGraph(HashMap::from([(
				vec3_not_nan!(1., 2., 3.),
				HashSet::from([]),
			)]));

			let node = graph.node(Vec3::new(1., 2., 3.));

			assert_eq!(Some(vec3_not_nan!(1., 2., 3.)), node);
		}

		#[test]
		fn get_closest_translation() {
			let graph = MeshGridGraph(HashMap::from([
				(
					vec3_not_nan!(10., 2., 3.),
					HashSet::from([vec3_not_nan!(20., 3., 4.)]),
				),
				(
					vec3_not_nan!(1., 2., 3.),
					HashSet::from([vec3_not_nan!(2., 30., 4.)]),
				),
			]));

			let node = graph.node(Vec3::new(2., 3., 4.));

			assert_eq!(Some(vec3_not_nan!(1., 2., 3.)), node);
		}
	}

	mod successors {
		use super::*;

		#[test]
		fn neighbors_as_successors() {
			let graph = MeshGridGraph(HashMap::from([(
				vec3_not_nan!(1., 2., 3.),
				HashSet::from([vec3_not_nan!(1., 2., 4.), vec3_not_nan!(1., 2., 5.)]),
			)]));

			let successors = graph.successors(&vec3_not_nan!(1., 2., 3.));

			assert_eq!(
				HashSet::from([vec3_not_nan!(1., 2., 4.), vec3_not_nan!(1., 2., 5.)]),
				successors.collect::<HashSet<_>>(),
			);
		}
	}

	mod los {
		use super::*;
		use testing::repeat_scope;

		#[test]
		fn los_to_self() {
			let graph = MeshGridGraph(HashMap::from([(
				vec3_not_nan!(1., 2., 3.),
				HashSet::from([]),
			)]));

			let los = graph.line_of_sight(&vec3_not_nan!(1., 2., 3.), &vec3_not_nan!(1., 2., 3.));

			assert!(los);
		}

		#[test]
		fn no_los_to_self_if_not_on_grid() {
			let graph = MeshGridGraph(HashMap::from([]));

			let los = graph.line_of_sight(&vec3_not_nan!(1., 2., 3.), &vec3_not_nan!(1., 2., 3.));

			assert!(!los);
		}

		#[test]
		fn los_to_neighbor() {
			let graph = MeshGridGraph(HashMap::from([
				(
					vec3_not_nan!(1., 2., 3.),
					HashSet::from([vec3_not_nan!(1., 2., 4.)]),
				),
				(
					vec3_not_nan!(1., 2., 4.),
					HashSet::from([vec3_not_nan!(1., 2., 3.)]),
				),
			]));

			let los = graph.line_of_sight(&vec3_not_nan!(1., 2., 3.), &vec3_not_nan!(1., 2., 4.));

			assert!(los);
		}

		#[test]
		fn no_los_when_not_connected() {
			let graph = MeshGridGraph(HashMap::from([
				(vec3_not_nan!(1., 2., 3.), HashSet::from([])),
				(vec3_not_nan!(1., 2., 4.), HashSet::from([])),
			]));

			let los = graph.line_of_sight(&vec3_not_nan!(1., 2., 3.), &vec3_not_nan!(1., 2., 4.));

			assert!(!los);
		}

		#[test]
		fn no_los_when_a_not_found() {
			let graph = MeshGridGraph(HashMap::from([(
				vec3_not_nan!(1., 2., 4.),
				HashSet::from([]),
			)]));

			let los = graph.line_of_sight(&vec3_not_nan!(1., 2., 3.), &vec3_not_nan!(1., 2., 4.));

			assert!(!los);
		}

		/// ```
		/// a — n1
		///   \ |  \
		///     n2 — b
		/// ```
		#[test]
		fn los_a_to_b_via_common_edge() {
			let a = vec3_not_nan!(0., 0., 0.);
			let n1 = vec3_not_nan!(1., 0., 0.);
			let n2 = vec3_not_nan!(1., 0., 1.);
			let b = vec3_not_nan!(2., 0., 1.);
			let graph = MeshGridGraph(HashMap::from([
				(a, HashSet::from([n1, n2])),
				(n1, HashSet::from([a, n2, b])),
				(n2, HashSet::from([a, n1, b])),
				(b, HashSet::from([n1, n2])),
			]));

			let los = graph.line_of_sight(&a, &b);

			assert!(los);
		}

		/// ```
		/// a —— n1
		/// | ## |
		/// n2 — b
		/// ```
		#[test]
		fn no_los_a_to_b_when_common_nodes_no_edge() {
			let a = vec3_not_nan!(0., 0., 0.);
			let n1 = vec3_not_nan!(1., 0., 0.);
			let n2 = vec3_not_nan!(0., 0., 1.);
			let b = vec3_not_nan!(1., 0., 1.);
			let graph = MeshGridGraph(HashMap::from([
				(a, HashSet::from([n1, n2])),
				(n1, HashSet::from([a, b])),
				(n2, HashSet::from([a, b])),
				(b, HashSet::from([n1, n2])),
			]));

			let los = graph.line_of_sight(&a, &b);

			assert!(!los);
		}

		/// ```
		/// a — b
		/// ```
		#[test]
		fn no_los_a_to_b_when_a_not_node() {
			let a = vec3_not_nan!(0., 0., 0.);
			let b = vec3_not_nan!(0., 0., 1.);

			// repeating due to hash set order randomness
			repeat_scope!(10, {
				let graph = MeshGridGraph(HashMap::from([(b, HashSet::from([a]))]));

				let los = graph.line_of_sight(&a, &b);

				assert!(!los);
			})
		}

		/// ```
		/// a — b
		/// ```
		#[test]
		fn no_los_a_to_b_when_b_not_node() {
			let a = vec3_not_nan!(0., 0., 0.);
			let b = vec3_not_nan!(0., 0., 1.);

			// repeating due to hash set order randomness
			repeat_scope!(10, {
				let graph = MeshGridGraph(HashMap::from([(a, HashSet::from([b]))]));

				let los = graph.line_of_sight(&a, &b);

				assert!(!los);
			})
		}

		/// ```
		/// a — n1 # #
		///   \ |  # #
		///     n2 — b
		/// ```
		#[test]
		fn no_los_a_to_b_via_common_edge_when_steep_corner_not_connected_to_b() {
			let a = vec3_not_nan!(0., 0., 0.);
			let n1 = vec3_not_nan!(1., 0., 0.);
			let n2 = vec3_not_nan!(1., 0., 1.);
			let b = vec3_not_nan!(2., 0., 1.);
			let graph = MeshGridGraph(HashMap::from([
				(a, HashSet::from([n1, n2])),
				(n1, HashSet::from([a, n2])),
				(n2, HashSet::from([a, n1, b])),
				(b, HashSet::from([n2])),
			]));

			let los = graph.line_of_sight(&a, &b);

			assert!(!los);
		}

		/// ```
		///     n3
		///   /
		/// a — n1
		///   \ |  \
		///     n2 — b
		/// ```
		#[test]
		fn los_a_to_b_via_common_edge_with_additional_node() {
			let a = vec3_not_nan!(0., 0., 0.);
			let n1 = vec3_not_nan!(1., 0., 0.);
			let n2 = vec3_not_nan!(1., 0., 1.);
			let n3 = vec3_not_nan!(1., 0., -1.);
			let b = vec3_not_nan!(2., 0., 1.);

			// repeating due to hash set order randomness
			repeat_scope!(10, {
				let graph = MeshGridGraph(HashMap::from([
					(a, HashSet::from([n1, n2, n3])),
					(n1, HashSet::from([a, n2, b])),
					(n2, HashSet::from([a, n1, b])),
					(n3, HashSet::from([a])),
					(b, HashSet::from([n1, n2])),
				]));

				let los = graph.line_of_sight(&a, &b);

				assert!(los);
			});
		}

		/// ```
		///     n2 — b
		///   / |  /
		/// a — n1
		///   \
		///     n3
		/// ```
		#[test]
		fn los_a_to_b_via_common_edge_with_additional_node_mirrored() {
			let a = vec3_not_nan!(0., 0., 0.);
			let n1 = vec3_not_nan!(1., 0., 0.);
			let n2 = vec3_not_nan!(1., 0., -1.);
			let n3 = vec3_not_nan!(1., 0., 1.);
			let b = vec3_not_nan!(2., 0., -1.);

			// repeating due to hash set order randomness
			repeat_scope!(10, {
				let graph = MeshGridGraph(HashMap::from([
					(a, HashSet::from([n1, n2, n3])),
					(n1, HashSet::from([a, n2, b])),
					(n2, HashSet::from([a, n1, b])),
					(n3, HashSet::from([a])),
					(b, HashSet::from([n1, n2])),
				]));

				let los = graph.line_of_sight(&a, &b);

				assert!(los);
			});
		}

		/// ```
		/// a — n1
		/// | \ |  \
		/// o — n2 — b
		/// ```
		#[test]
		fn los_a_to_b_via_common_edge_when_a_has_other_nodes() {
			let a = vec3_not_nan!(0., 0., 0.);
			let o = vec3_not_nan!(0., 0., 1.);
			let n1 = vec3_not_nan!(1., 0., 0.);
			let n2 = vec3_not_nan!(1., 0., 1.);
			let b = vec3_not_nan!(2., 0., 0.);

			// repeating due to hash set order randomness
			repeat_scope!(10, {
				let graph = MeshGridGraph(HashMap::from([
					(a, HashSet::from([n1, n2])),
					(o, HashSet::from([a, n2])),
					(n1, HashSet::from([a, n2, b])),
					(n2, HashSet::from([a, n1, b])),
					(b, HashSet::from([n1, n2])),
				]));

				let los = graph.line_of_sight(&a, &b);

				assert!(los);
			})
		}

		/// ```
		///     n1
		///   / ## \
		/// a — n2 — b
		///   \ |  /
		///     n3
		/// ```
		#[test]
		fn los_a_to_b_via_common_edge_when_multiple_edges_available() {
			let a = vec3_not_nan!(0., 0., 0.);
			let b = vec3_not_nan!(2., 0., 0.);
			let n1 = vec3_not_nan!(1., 0., -1.);
			let n2 = vec3_not_nan!(1., 0., 0.);
			let n3 = vec3_not_nan!(1., 0., 1.);

			// repeating due to hash set order randomness
			repeat_scope!(10, {
				let graph = MeshGridGraph(HashMap::from([
					(a, HashSet::from([n1, n2, n3])),
					(n1, HashSet::from([a, b])),
					(n2, HashSet::from([a, n3, b])),
					(n3, HashSet::from([a, n2, b])),
					(b, HashSet::from([n1, n2, n3])),
				]));

				let los = graph.line_of_sight(&a, &b);

				assert!(los);
			})
		}

		/// ```
		/// a — n1
		///   \ |  \
		///     n2 — i1
		///        \ |  \
		///          i2 — b
		/// ```
		#[test]
		fn los_a_to_b_distant() {
			let a = vec3_not_nan!(0., 0., 0.);
			let b = vec3_not_nan!(3., 0., 2.);
			let n1 = vec3_not_nan!(1., 0., 0.);
			let n2 = vec3_not_nan!(1., 0., 1.);
			let i1 = vec3_not_nan!(2., 0., 1.);
			let i2 = vec3_not_nan!(2., 0., 2.);
			let graph = MeshGridGraph(HashMap::from([
				(a, HashSet::from([n1, n2])),
				(n1, HashSet::from([a, n2, i1])),
				(n2, HashSet::from([a, n1, i1, i2])),
				(i1, HashSet::from([n1, n2, i2, b])),
				(i2, HashSet::from([n2, i1, b])),
				(b, HashSet::from([i1, i2])),
			]));

			let los = graph.line_of_sight(&a, &b);

			assert!(los);
		}
	}
}
