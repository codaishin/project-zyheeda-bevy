use super::*;

#[derive(Debug, PartialEq, Eq, Hash)]
pub(super) struct Edge((NodeId, VecNotNan<3>), (NodeId, VecNotNan<3>));

impl Edge {
	const TEST_TOLERANCE: f32 = 1e-6;

	pub(super) fn uniform(a: (NodeId, VecNotNan<3>), b: (NodeId, VecNotNan<3>)) -> Self {
		match a.0.cmp(&b.0) {
			Ordering::Less => Self(a, b),
			_ => Self(b, a),
		}
	}

	pub(super) fn ids(&self) -> (NodeId, NodeId) {
		(self.0.0, self.1.0)
	}

	pub(super) fn crossed_by(&self, a: Vec3, b: Vec3) -> bool {
		let Ok(to_edge0) = Dir3::try_from(Vec3::from(self.0.1) - a) else {
			return false;
		};
		let Ok(to_edge1) = Dir3::try_from(Vec3::from(self.1.1) - a) else {
			return false;
		};
		let Ok(ab) = Dir3::try_from(b - a) else {
			return false;
		};

		let dot_wedge = to_edge0.dot(*to_edge1);
		let dot_ab0 = ab.dot(*to_edge0);
		let dot_ab1 = ab.dot(*to_edge1);

		dot_wedge < f32::min(dot_ab0, dot_ab1)
	}

	pub(super) fn spans_obtuse_angle_to(&self, v: Vec3) -> bool {
		let Ok(to_edge0) = Dir3::try_from(Vec3::from(self.0.1) - v) else {
			return false;
		};
		let Ok(to_edge1) = Dir3::try_from(Vec3::from(self.1.1) - v) else {
			return false;
		};

		to_edge0.dot(*to_edge1) <= Self::TEST_TOLERANCE
	}
}

impl From<Edge> for TriangleEdgeError {
	fn from(Edge((.., a), (.., b)): Edge) -> Self {
		Self(a, b)
	}
}
