use super::*;
use core::f32;
use std::ops::Add;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(super) struct Clearance(f32);

impl Clearance {
	pub(super) const NONE: Self = Self(0.0);
	pub(super) const INFINITY: Self = Self(f32::INFINITY);

	pub(super) fn is_zero_or_smaller_than(&self, other: impl Into<Clearance>) -> bool {
		self == &Self::NONE || self.0 < other.into().0
	}
}

impl Eq for Clearance {}

impl From<Units> for Clearance {
	fn from(value: Units) -> Self {
		Self(*value)
	}
}

impl Add<Units> for Clearance {
	type Output = Self;

	fn add(self, rhs: Units) -> Self::Output {
		Clearance(self.0 + *rhs)
	}
}

pub(super) struct SetClearance {
	heap: BinaryHeap<SmallestClearance>,
}

impl SetClearance {
	#[must_use]
	pub(super) fn from_boundary(boundary: impl IntoIterator<Item = NodeId>) -> Self {
		let heap = boundary
			.into_iter()
			.map(|node| SmallestClearance {
				clearance: Clearance::NONE,
				node,
			})
			.collect();

		Self { heap }
	}

	pub(super) fn next_step(&mut self) -> Option<Step<'_>> {
		let smallest = self.heap.pop()?;

		Some(Step {
			smallest,
			heap: &mut self.heap,
		})
	}
}

pub(crate) struct Step<'a> {
	smallest: SmallestClearance,
	heap: &'a mut BinaryHeap<SmallestClearance>,
}

impl Step<'_> {
	pub(super) fn process(self, graph: &mut MeshGridGraph) {
		if graph.clearance[*self.smallest.node].0 < self.smallest.clearance.0 {
			return;
		}

		graph.clearance[*self.smallest.node] = self.smallest.clearance;

		let GroundPosition(pos) = graph.ground_position(&self.smallest.node);

		for successor in graph.successors(&self.smallest.node) {
			let GroundPosition(s_pos) = graph.ground_position(&successor);
			let s_distance = Units::from((s_pos - pos).length());
			let s_clearance = self.smallest.clearance + s_distance;

			if graph.clearance[*successor].0 <= s_clearance.0 {
				continue;
			}

			self.heap.push(SmallestClearance {
				clearance: s_clearance,
				node: successor,
			});
		}
	}
}

#[derive(PartialEq, Eq)]
struct SmallestClearance {
	pub(super) clearance: Clearance,
	pub(super) node: NodeId,
}

impl Ord for SmallestClearance {
	fn cmp(&self, other: &Self) -> Ordering {
		other
			.clearance
			.0
			.total_cmp(&self.clearance.0)
			.then_with(|| other.node.cmp(&self.node))
	}
}

impl PartialOrd for SmallestClearance {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::mesh_grid_graph::test::neighbors;
	use common::vec_not_nan;

	#[test]
	fn set_first_border_node() {
		let mut graph = MeshGridGraph {
			vertices: vec![
				vec_not_nan!(0., 0., 0.),
				vec_not_nan!(0., 0., 1.),
				vec_not_nan!(1., 0., 0.),
				vec_not_nan!(1., 0., 1.),
			],
			neighbors: neighbors![[], [], [], []],
			clearance: vec![
				Clearance::INFINITY,
				Clearance::INFINITY,
				Clearance::INFINITY,
				Clearance::INFINITY,
			],
			..default()
		};
		let mut set_clearance = SetClearance::from_boundary([NodeId(0), NodeId(2)]);

		set_clearance.next_step().unwrap().process(&mut graph);

		assert_eq!(
			MeshGridGraph {
				vertices: vec![
					vec_not_nan!(0., 0., 0.),
					vec_not_nan!(0., 0., 1.),
					vec_not_nan!(1., 0., 0.),
					vec_not_nan!(1., 0., 1.),
				],
				neighbors: neighbors![[], [], [], []],
				clearance: vec![
					Clearance::NONE,
					Clearance::INFINITY,
					Clearance::INFINITY,
					Clearance::INFINITY,
				],
				..default()
			},
			graph,
		);
	}

	#[test]
	fn set_all_border_nodes() {
		let mut graph = MeshGridGraph {
			vertices: vec![
				vec_not_nan!(0., 0., 0.),
				vec_not_nan!(0., 0., 1.),
				vec_not_nan!(1., 0., 0.),
				vec_not_nan!(1., 0., 1.),
			],
			neighbors: neighbors![[], [], [], []],
			clearance: vec![
				Clearance::INFINITY,
				Clearance::INFINITY,
				Clearance::INFINITY,
				Clearance::INFINITY,
			],
			..default()
		};
		let mut set_clearance = SetClearance::from_boundary([NodeId(0), NodeId(2)]);

		set_clearance.next_step().unwrap().process(&mut graph);
		set_clearance.next_step().unwrap().process(&mut graph);

		assert_eq!(
			MeshGridGraph {
				vertices: vec![
					vec_not_nan!(0., 0., 0.),
					vec_not_nan!(0., 0., 1.),
					vec_not_nan!(1., 0., 0.),
					vec_not_nan!(1., 0., 1.),
				],
				neighbors: neighbors![[], [], [], []],
				clearance: vec![
					Clearance::NONE,
					Clearance::INFINITY,
					Clearance::NONE,
					Clearance::INFINITY,
				],
				..default()
			},
			graph,
		);
	}

	#[test]
	fn set_neighbors() {
		let mut graph = MeshGridGraph {
			vertices: vec![
				vec_not_nan!(0., 0., 0.),
				vec_not_nan!(0., 0., 1.),
				vec_not_nan!(1., 0., 0.),
				vec_not_nan!(1., 0., 1.),
			],
			neighbors: neighbors![[1, 3], [0], [], []],
			clearance: vec![
				Clearance::INFINITY,
				Clearance::INFINITY,
				Clearance::INFINITY,
				Clearance::INFINITY,
			],
			..default()
		};
		let mut set_clearance = SetClearance::from_boundary([NodeId(0), NodeId(2)]);

		set_clearance.next_step().unwrap().process(&mut graph);
		set_clearance.next_step().unwrap().process(&mut graph);
		set_clearance.next_step().unwrap().process(&mut graph);
		set_clearance.next_step().unwrap().process(&mut graph);

		assert_eq!(
			MeshGridGraph {
				vertices: vec![
					vec_not_nan!(0., 0., 0.),
					vec_not_nan!(0., 0., 1.),
					vec_not_nan!(1., 0., 0.),
					vec_not_nan!(1., 0., 1.),
				],
				neighbors: neighbors![[1, 3], [0], [], []],
				clearance: vec![
					Clearance::NONE,
					Clearance::from(Units::from(1.)),
					Clearance::NONE,
					Clearance::from(Units::from(f32::sqrt(2.))),
				],
				..default()
			},
			graph,
		);
	}

	#[test]
	fn set_neighbors_from_all_borders() {
		let mut graph = MeshGridGraph {
			vertices: vec![
				vec_not_nan!(0., 0., 0.),
				vec_not_nan!(0., 0., 1.),
				vec_not_nan!(1., 0., 0.),
				vec_not_nan!(1., 0., 1.),
			],
			neighbors: neighbors![[1, 3], [0, 2], [1, 3], [0, 2]],
			clearance: vec![
				Clearance::INFINITY,
				Clearance::INFINITY,
				Clearance::INFINITY,
				Clearance::INFINITY,
			],
			..default()
		};
		let mut set_clearance = SetClearance::from_boundary([NodeId(0), NodeId(2)]);

		set_clearance.next_step().unwrap().process(&mut graph);
		set_clearance.next_step().unwrap().process(&mut graph);
		set_clearance.next_step().unwrap().process(&mut graph);
		set_clearance.next_step().unwrap().process(&mut graph);
		set_clearance.next_step().unwrap().process(&mut graph);
		set_clearance.next_step().unwrap().process(&mut graph);

		assert_eq!(
			MeshGridGraph {
				vertices: vec![
					vec_not_nan!(0., 0., 0.),
					vec_not_nan!(0., 0., 1.),
					vec_not_nan!(1., 0., 0.),
					vec_not_nan!(1., 0., 1.),
				],
				neighbors: neighbors![[1, 3], [0, 2], [1, 3], [0, 2]],
				clearance: vec![
					Clearance::NONE,
					Clearance::from(Units::from(1.)),
					Clearance::NONE,
					Clearance::from(Units::from(1.)),
				],
				..default()
			},
			graph,
		);
	}

	#[test]
	fn set_neighbors_of_neighbors_of_border() {
		let mut graph = MeshGridGraph {
			vertices: vec![
				vec_not_nan!(0., 0., 0.),
				vec_not_nan!(0., 0., 1.),
				vec_not_nan!(1., 0., 0.),
				vec_not_nan!(1., 0., 1.),
			],
			neighbors: neighbors![[1], [0, 2], [1, 3], [2]],
			clearance: vec![
				Clearance::INFINITY,
				Clearance::INFINITY,
				Clearance::INFINITY,
				Clearance::INFINITY,
			],
			..default()
		};
		let mut set_clearance = SetClearance::from_boundary([NodeId(0)]);

		while let Some(step) = set_clearance.next_step() {
			step.process(&mut graph);
		}

		assert_eq!(
			MeshGridGraph {
				vertices: vec![
					vec_not_nan!(0., 0., 0.),
					vec_not_nan!(0., 0., 1.),
					vec_not_nan!(1., 0., 0.),
					vec_not_nan!(1., 0., 1.),
				],
				neighbors: neighbors![[1], [0, 2], [1, 3], [2]],
				clearance: vec![
					Clearance::NONE,
					Clearance::from(Units::from(1.)),
					Clearance::from(Units::from(1. + f32::sqrt(2.))),
					Clearance::from(Units::from(2. + f32::sqrt(2.))),
				],
				..default()
			},
			graph,
		);
	}

	#[test]
	fn preemptive_disregard_of_stale_values() {
		let mut graph = MeshGridGraph {
			vertices: vec![
				vec_not_nan!(0., 0., 0.),
				vec_not_nan!(0., 0., 1.),
				vec_not_nan!(1., 0., 0.),
				vec_not_nan!(1., 0., 1.),
			],
			neighbors: neighbors![[1, 3], [0, 2], [1, 3], [0, 2]],
			clearance: vec![
				Clearance::INFINITY,
				Clearance::INFINITY,
				Clearance::INFINITY,
				Clearance::INFINITY,
			],
			..default()
		};
		let mut set_clearance = SetClearance::from_boundary([NodeId(0), NodeId(2)]);

		set_clearance.next_step().unwrap().process(&mut graph);
		set_clearance.next_step().unwrap().process(&mut graph);
		set_clearance.next_step().unwrap().process(&mut graph);
		set_clearance.next_step().unwrap().process(&mut graph);
		set_clearance.next_step().unwrap().process(&mut graph);
		set_clearance.next_step().unwrap().process(&mut graph);

		assert!(set_clearance.next_step().is_none());
	}

	#[test]
	fn use_best_value_first() {
		let mut graph = MeshGridGraph {
			vertices: vec![
				vec_not_nan!(0., 0., 0.),
				vec_not_nan!(0., 0., 1.),
				vec_not_nan!(0., 0., 2.),
				vec_not_nan!(0., 0., 3.),
			],
			neighbors: neighbors![[3], [3], [3], [0, 1, 2]],
			clearance: vec![
				Clearance::INFINITY,
				Clearance::INFINITY,
				Clearance::INFINITY,
				Clearance::INFINITY,
			],
			..default()
		};
		let mut set_clearance = SetClearance::from_boundary([NodeId(0), NodeId(1), NodeId(2)]);

		set_clearance.next_step().unwrap().process(&mut graph);
		set_clearance.next_step().unwrap().process(&mut graph);
		set_clearance.next_step().unwrap().process(&mut graph);
		set_clearance.next_step().unwrap().process(&mut graph);

		assert_eq!(
			MeshGridGraph {
				vertices: vec![
					vec_not_nan!(0., 0., 0.),
					vec_not_nan!(0., 0., 1.),
					vec_not_nan!(0., 0., 2.),
					vec_not_nan!(0., 0., 3.),
				],
				neighbors: neighbors![[3], [3], [3], [0, 1, 2]],
				clearance: vec![
					Clearance::NONE,
					Clearance::NONE,
					Clearance::NONE,
					Clearance::from(Units::from(1.)),
				],
				..default()
			},
			graph,
		);
	}
}
