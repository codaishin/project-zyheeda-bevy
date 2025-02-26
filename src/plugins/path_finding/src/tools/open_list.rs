use super::nav_grid_node::NavGridNode;
use std::{
	cmp::{Ordering, Reverse},
	collections::BinaryHeap,
};

pub(crate) struct OpenList<'a> {
	end: NavGridNode,
	heap: BinaryHeap<Reverse<Node>>,
	dist_f: &'a dyn Fn(NavGridNode, NavGridNode) -> f32,
}

impl<'a> OpenList<'a> {
	pub(crate) fn new(
		start: NavGridNode,
		end: NavGridNode,
		dist_f: &'a dyn Fn(NavGridNode, NavGridNode) -> f32,
	) -> Self {
		let f = dist_f(start, end);
		OpenList {
			end,
			dist_f,
			heap: BinaryHeap::from([Reverse(Node { node: start, f })]),
		}
	}

	pub(crate) fn pop_lowest_f(&mut self) -> Option<NavGridNode> {
		self.heap.pop().map(|Reverse(Node { node, .. })| node)
	}

	pub(crate) fn push(&mut self, node: NavGridNode, g: f32) {
		let f = g + (self.dist_f)(node, self.end);
		self.heap.push(Reverse(Node { node, f }));
	}
}

#[derive(Debug, PartialEq)]
struct Node {
	node: NavGridNode,
	f: f32,
}

impl Eq for Node {}

impl PartialOrd for Node {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for Node {
	fn cmp(&self, other: &Self) -> Ordering {
		let Some(c_f) = self.f.partial_cmp(&other.f) else {
			panic!(
				"tried to compare {:?} with {:?} (f values are not comparable)",
				self, other
			);
		};
		c_f.then_with(|| self.node.cmp(&other.node))
	}
}
