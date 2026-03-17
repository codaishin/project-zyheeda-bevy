use super::*;

const MIN_DIFF_EPSILON: f32 = 1e-6;

pub(super) struct IterLine<'a> {
	graph: &'a MeshGridGraph,
	seen: HashSet<NodeId>,
	open: VecDeque<NodeId>,
	target_pos: Vec3,
	required_clearance: Units,
}

impl<'a> IterLine<'a> {
	pub(super) fn new(
		origin: NodeId,
		target: NodeId,
		graph: &'a MeshGridGraph,
		required_clearance: Units,
	) -> Self {
		let GroundPosition(target_pos) = graph.ground_position(&target);

		line::IterLine {
			graph,
			seen: HashSet::from([origin]),
			open: VecDeque::from([origin]),
			target_pos,
			required_clearance,
		}
	}
}

impl Iterator for IterLine<'_> {
	type Item = NodeId;

	fn next(&mut self) -> Option<Self::Item> {
		let next = self.open.pop_front()?;

		if self.graph.is_obstacle(&next, self.required_clearance) {
			return None;
		}

		let GroundPosition(pos) = self.graph.ground_position(&next);
		let dir = match Dir3::try_from(self.target_pos - pos) {
			Err(InvalidDirectionError::Zero) => {
				self.open.clear();
				return Some(next);
			}
			Err(_) => return None,
			Ok(dir) => dir,
		};

		let mut forward_successors = BinaryHeap::new();

		for node in self.graph.successors(&next) {
			let GroundPosition(n_pos) = self.graph.ground_position(&node);
			let dot = Dir3::try_from(n_pos - pos)
				.map(|n_dir| n_dir.dot(*dir))
				.unwrap_or(-1.);

			if dot >= -MIN_DIFF_EPSILON {
				forward_successors.push(ForwardSuccessor {
					node,
					forward_dot: dot,
				});
			}
		}

		let mut it = IterSuccessorsOrdered(forward_successors).peekable();

		let it = match it.peek() {
			Some(first) if (first.forward_dot - 1.).abs() <= MIN_DIFF_EPSILON => it.take(1),
			_ => it.take(2),
		};

		for ForwardSuccessor { node, .. } in it {
			if !self.seen.insert(node) {
				continue;
			}

			self.open.push_back(node);
		}

		Some(next)
	}
}

#[derive(PartialEq, Debug)]
struct ForwardSuccessor {
	forward_dot: f32,
	node: NodeId,
}

impl Eq for ForwardSuccessor {}

impl Ord for ForwardSuccessor {
	fn cmp(&self, other: &Self) -> Ordering {
		self.forward_dot
			.total_cmp(&other.forward_dot)
			.then_with(|| other.node.cmp(&self.node))
	}
}

impl PartialOrd for ForwardSuccessor {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

struct IterSuccessorsOrdered(BinaryHeap<ForwardSuccessor>);

impl Iterator for IterSuccessorsOrdered {
	type Item = ForwardSuccessor;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.pop()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::mesh_grid_graph::test::neighbors;
	use common::vec3_not_nan;

	/// ```
	/// a — n1
	///   \ |  \
	///     n2 — n3
	///        \ |  \
	///          n4 — b
	/// ```
	#[test]
	fn line_walks_towards_target() {
		let a = vec3_not_nan!(0., 0., 0.);
		let n1 = vec3_not_nan!(1., 0., 0.);
		let n2 = vec3_not_nan!(1., 0., 1.);
		let n3 = vec3_not_nan!(2., 0., 1.);
		let n4 = vec3_not_nan!(2., 0., 2.);
		let b = vec3_not_nan!(3., 0., 2.);
		let graph = MeshGridGraph {
			vertices: vec![a, n1, n2, n3, n4, b],
			neighbors: neighbors![
				[1, 2],
				[0, 2, 3],
				[0, 1, 3, 4],
				[1, 2, 4, 5],
				[2, 3, 5],
				[3, 4],
			],
			clearance: vec![Clearance::from(Units::from(1.0)); 6],
		};

		let line = IterLine::new(NodeId(0), NodeId(5), &graph, Units::ZERO);

		assert_eq!(
			vec![
				NodeId(0),
				NodeId(2),
				NodeId(1),
				NodeId(4),
				NodeId(3),
				NodeId(5),
			],
			line.collect::<Vec<_>>(),
		);
	}

	/// ```
	/// 00 — 01 — 02 — 03 — 04
	/// |  X |  X |  X |  X |
	/// 05 —a06 — 07 — 08 — 09
	/// |  X |  X |  X |  X |
	/// 10 — 11 — 12 —b13 — 14
	/// |  X |  X |  X |  X |
	/// 15 — 16 — 17 — 18 — 19
	/// ```
	#[test]
	fn line_uses_closest_wedge_towards_target_with_surrounding_nodes() {
		let vertices = [
			vec3_not_nan!(0., 0., 0.),
			vec3_not_nan!(1., 0., 0.),
			vec3_not_nan!(2., 0., 0.),
			vec3_not_nan!(3., 0., 0.),
			vec3_not_nan!(4., 0., 0.),
			vec3_not_nan!(0., 0., 1.),
			vec3_not_nan!(1., 0., 1.),
			vec3_not_nan!(2., 0., 1.),
			vec3_not_nan!(3., 0., 1.),
			vec3_not_nan!(4., 0., 1.),
			vec3_not_nan!(0., 0., 2.),
			vec3_not_nan!(1., 0., 2.),
			vec3_not_nan!(2., 0., 2.),
			vec3_not_nan!(3., 0., 2.),
			vec3_not_nan!(4., 0., 2.),
			vec3_not_nan!(0., 0., 3.),
			vec3_not_nan!(1., 0., 3.),
			vec3_not_nan!(2., 0., 3.),
			vec3_not_nan!(3., 0., 3.),
			vec3_not_nan!(4., 0., 3.),
		];
		let graph = MeshGridGraph {
			vertices: Vec::from(vertices),
			neighbors: neighbors![
				// row 0
				[1, 5, 6],
				[0, 2, 5, 6, 7],
				[1, 3, 6, 7, 8],
				[2, 4, 7, 8, 9],
				[3, 8, 9],
				// row 1
				[0, 1, 6, 10, 11],
				[0, 1, 2, 5, 7, 10, 11, 12],
				[1, 2, 3, 6, 7, 11, 12, 13],
				[2, 3, 4, 7, 9, 12, 13, 14],
				[3, 4, 8, 13, 14],
				// row 2
				[5, 6, 11, 15, 16],
				[5, 6, 7, 10, 12, 15, 16, 17],
				[6, 7, 8, 11, 13, 16, 17, 18],
				[7, 8, 9, 12, 14, 17, 18, 19],
				[8, 9, 13, 18, 19],
				// row 3
				[10, 11, 16, 17],
				[10, 11, 12, 15, 17],
				[11, 12, 13, 16, 18],
				[12, 13, 14, 17, 19],
				[13, 14, 18],
			],
			clearance: vec![Clearance::from(Units::from(1.0)); 20],
		};

		let line = IterLine::new(NodeId(6), NodeId(13), &graph, Units::ZERO);

		assert_eq!(
			vec![NodeId(6), NodeId(12), NodeId(7), NodeId(13)],
			line.collect::<Vec<_>>(),
		);
	}

	/// ```
	///    n1 — b
	///   /
	///  a
	///   \
	///    n2 — n3(stale)
	/// ```
	#[test]
	fn line_disregards_stale_nodes() {
		let a = vec3_not_nan!(0., 0., 0.);
		let n1 = vec3_not_nan!(1., 0., 0.3);
		let n2 = vec3_not_nan!(1., 0., -0.3);
		let n3 = vec3_not_nan!(2., 0., -0.5);
		let b = vec3_not_nan!(2., 0., 0.);
		let graph = MeshGridGraph {
			vertices: vec![a, n1, n2, n3, b],
			neighbors: neighbors![[1, 2], [0, 2, 4], [0, 1, 3], [2], [1]],
			clearance: vec![Clearance::from(Units::from(1.0)); 5],
		};

		let line = IterLine::new(NodeId(0), NodeId(4), &graph, Units::ZERO);

		assert_eq!(line.last(), Some(NodeId(4)));
	}

	/// ```
	/// a — n1
	///   \ |  \
	///     n2 — n3
	///        \ |  \
	///          n4 — b
	/// ```
	#[test]
	fn stop_line_when_clearance_too_low() {
		let a = vec3_not_nan!(0., 0., 0.);
		let n1 = vec3_not_nan!(1., 0., 0.);
		let n2 = vec3_not_nan!(1., 0., 1.);
		let n3 = vec3_not_nan!(2., 0., 1.);
		let n4 = vec3_not_nan!(2., 0., 2.);
		let b = vec3_not_nan!(3., 0., 2.);
		let graph = MeshGridGraph {
			vertices: vec![a, n1, n2, n3, n4, b],
			neighbors: neighbors![
				[1, 2],
				[0, 2, 3],
				[0, 1, 3, 4],
				[1, 2, 4, 5],
				[2, 3, 5],
				[3, 4],
			],
			clearance: vec![
				Clearance::from(Units::from(1.0)),
				Clearance::from(Units::from(1.0)),
				Clearance::from(Units::from(1.0)),
				Clearance::from(Units::from(0.9)),
				Clearance::from(Units::from(1.0)),
				Clearance::from(Units::from(1.0)),
			],
		};

		let line = IterLine::new(NodeId(0), NodeId(5), &graph, Units::from_u8(1));

		assert_eq!(
			vec![NodeId(0), NodeId(2), NodeId(1), NodeId(4)],
			line.collect::<Vec<_>>()
		);
	}

	/// ```
	///     n1
	///   / |  \
	/// a   |    b
	///   \ |  /
	///     n2
	/// ```
	#[test]
	fn explore_all_nodes_if_multiple_are_valid() {
		let a = vec3_not_nan!(0., 0., 0.);
		let n1 = vec3_not_nan!(1., 0., -1.);
		let n2 = vec3_not_nan!(1., 0., 1.);
		let b = vec3_not_nan!(2., 0., 0.);
		let graph = MeshGridGraph {
			vertices: vec![a, n1, n2, b],
			neighbors: neighbors![[1, 2], [0, 2, 3], [0, 1, 3], [1, 2]],
			clearance: vec![
				Clearance::from(Units::from(1.0)),
				Clearance::from(Units::from(1.0)),
				Clearance::from(Units::from(1.0)),
				Clearance::NONE,
			],
		};

		let line = IterLine::new(NodeId(0), NodeId(3), &graph, Units::from_u8(0));

		assert_eq!(
			vec![NodeId(0), NodeId(1), NodeId(2)],
			line.collect::<Vec<_>>(),
		);
	}

	/// ```
	///     n1
	///   / |  \
	/// a   |    b
	///   \ |  /
	///     n2
	/// ```
	#[test]
	fn explore_all_nodes_if_multiple_are_valid_allowing_for_some_epsilon() {
		let a = vec3_not_nan!(0., 0., 0.);
		let n1 = vec3_not_nan!(1., 0., -1.0000001);
		let n2 = vec3_not_nan!(1., 0., 1.);
		let b = vec3_not_nan!(2., 0., 0.);
		let graph = MeshGridGraph {
			vertices: vec![a, n1, n2, b],
			neighbors: neighbors![[1, 2], [0, 2, 3], [0, 1, 3], [1, 2]],
			clearance: vec![
				Clearance::from(Units::from(1.0)),
				Clearance::from(Units::from(1.0)),
				Clearance::from(Units::from(1.0)),
				Clearance::NONE,
			],
		};

		let line = IterLine::new(NodeId(0), NodeId(3), &graph, Units::from_u8(0));

		assert_eq!(
			vec![NodeId(0), NodeId(2), NodeId(1)],
			line.collect::<Vec<_>>(),
		);
	}

	/// ```
	///     n1
	///   / |  \
	/// a   |    n3 — b
	///   \ |  /
	///     n2
	/// ```
	#[test]
	fn do_not_revisit_nodes() {
		let a = vec3_not_nan!(0., 0., 0.);
		let n1 = vec3_not_nan!(1., 0., -1.);
		let n2 = vec3_not_nan!(1., 0., 1.);
		let n3 = vec3_not_nan!(2., 0., 0.);
		let b = vec3_not_nan!(3., 0., 0.);
		let graph = MeshGridGraph {
			vertices: vec![a, n1, n2, n3, b],
			neighbors: neighbors![[1, 2], [0, 2, 3], [0, 1, 3], [1, 2, 4], [3]],
			clearance: vec![Clearance::from(Units::from(1.0)); 5],
		};

		let line = IterLine::new(NodeId(0), NodeId(4), &graph, Units::ZERO);

		assert_eq!(
			vec![NodeId(0), NodeId(1), NodeId(2), NodeId(3), NodeId(4)],
			line.collect::<Vec<_>>(),
		);
	}

	/// ```
	/// a — n1
	///   \ |  \
	///     n2 — n3
	///        \ |  \
	///          n4 — b
	/// ```
	#[test]
	fn no_line_when_no_clearance_even_when_zero_clearance_required() {
		let a = vec3_not_nan!(0., 0., 0.);
		let n1 = vec3_not_nan!(1., 0., 0.);
		let n2 = vec3_not_nan!(1., 0., 1.);
		let n3 = vec3_not_nan!(2., 0., 1.);
		let n4 = vec3_not_nan!(2., 0., 2.);
		let b = vec3_not_nan!(3., 0., 2.);
		let graph = MeshGridGraph {
			vertices: vec![a, n1, n2, n3, n4, b],
			neighbors: neighbors![
				[1, 2],
				[0, 2, 3],
				[0, 1, 3, 4],
				[1, 2, 4, 5],
				[2, 3, 5],
				[3, 4],
			],
			clearance: vec![
				Clearance::NONE,
				Clearance::NONE,
				Clearance::NONE,
				Clearance::NONE,
				Clearance::NONE,
				Clearance::NONE,
			],
		};

		let line = IterLine::new(NodeId(0), NodeId(5), &graph, Units::ZERO);

		assert_eq!(vec![] as Vec<NodeId>, line.collect::<Vec<_>>());
	}
}
