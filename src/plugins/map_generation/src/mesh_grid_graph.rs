use crate::{
	systems::spawn_grid::TryFromTriangles,
	traits::to_subdivided::{SubdivisionError, ToSubdivided},
};
use bevy::prelude::*;
use common::tools::vec_not_nan::VecNotNan;
use std::{
	cmp::Ordering,
	collections::{HashMap, HashSet},
	fmt::{Debug, Display},
	hash::Hash,
};

#[derive(Debug, PartialEq, Clone, Default)]
pub struct MeshGridGraph(HashMap<VecNotNan<3>, HashSet<VecNotNan<3>>>);

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
		let mut graph = HashMap::<VecNotNan<3>, HashSet<VecNotNan<3>>>::default();
		let mut nodes_facing_same_edge = HashMap::<FacedEdge, Vec<VecNotNan<3>>>::default();

		for [a, b, c] in triangles {
			graph.entry(a).or_default().extend([b, c]);
			nodes_facing_same_edge
				.entry(FacedEdge::uniform(b, c))
				.or_default()
				.push(a);

			graph.entry(b).or_default().extend([a, c]);
			nodes_facing_same_edge
				.entry(FacedEdge::uniform(a, c))
				.or_default()
				.push(b);

			graph.entry(c).or_default().extend([a, b]);
			nodes_facing_same_edge
				.entry(FacedEdge::uniform(a, b))
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

#[derive(Debug, PartialEq)]
pub(crate) struct TriangleEdgeError(VecNotNan<3>, VecNotNan<3>);

impl From<FacedEdge> for TriangleEdgeError {
	fn from(FacedEdge(a, b): FacedEdge) -> Self {
		Self(a, b)
	}
}

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

#[derive(PartialEq, Eq, Hash)]
struct FacedEdge(VecNotNan<3>, VecNotNan<3>);

impl FacedEdge {
	fn uniform(a: VecNotNan<3>, b: VecNotNan<3>) -> Self {
		match a.cmp(&b) {
			Ordering::Less => Self(a, b),
			_ => Self(b, a),
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use common::vec3_not_nan;
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
