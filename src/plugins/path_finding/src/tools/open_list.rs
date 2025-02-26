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
		let heap = BinaryHeap::from([Reverse(Node {
			node: start,
			f: dist_f(start, end),
		})]);
		OpenList { end, dist_f, heap }
	}

	pub(crate) fn pop_lowest_f(&mut self) -> Option<NavGridNode> {
		self.heap.pop().map(|Reverse(Node { node, .. })| node)
	}

	pub(crate) fn push(&mut self, node: NavGridNode, g: f32) {
		let f = g + self.dist(node, self.end);
		self.heap.push(Reverse(Node { node, f }));
	}

	fn dist(&self, a: NavGridNode, b: NavGridNode) -> f32 {
		(self.dist_f)(a, b)
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

#[cfg(test)]
mod tests {
	use super::*;
	use mockall::{automock, predicate::eq};

	#[automock]
	trait _DistF {
		fn call(&self, a: NavGridNode, b: NavGridNode) -> f32;
	}

	macro_rules! new_dist_f {
		($setup:expr) => {{
			let mut mock = Mock_DistF::default();
			$setup(&mut mock);
			move |a, b| mock.call(a, b)
		}};
	}

	#[test]
	fn new() {
		let start = NavGridNode { x: 1, y: 2 };
		let end = NavGridNode { x: 3, y: 4 };
		let dist_f = new_dist_f!(|mock: &mut Mock_DistF| {
			mock.expect_call()
				.times(1)
				.with(eq(start), eq(end))
				.return_const(42.);
		});

		let mut list = OpenList::new(start, end, &dist_f);

		assert_eq!(Some(start), list.pop_lowest_f());
	}

	#[test]
	fn pop_by_lowest_f_value() {
		let a = NavGridNode { x: 1, y: 2 };
		let b = NavGridNode { x: 2, y: 3 };
		let end = NavGridNode { x: 3, y: 4 };
		let dist_f = new_dist_f!(|mock: &mut Mock_DistF| {
			mock.expect_call().with(eq(a), eq(end)).return_const(42.);
			mock.expect_call().with(eq(b), eq(end)).return_const(11.);
		});
		let mut list = OpenList::new(a, end, &dist_f);
		list.push(b, 0.);

		let nodes = [list.pop_lowest_f(), list.pop_lowest_f()];

		assert_eq!([Some(b), Some(a)], nodes);
	}

	#[test]
	fn pop_by_lowest_f_value_combined_with_g() {
		let a = NavGridNode { x: 1, y: 2 };
		let b = NavGridNode { x: 2, y: 3 };
		let end = NavGridNode { x: 3, y: 4 };
		let dist_f = new_dist_f!(|mock: &mut Mock_DistF| {
			mock.expect_call().with(eq(a), eq(end)).return_const(12.);
			mock.expect_call().with(eq(b), eq(end)).return_const(11.);
		});
		let mut list = OpenList::new(a, end, &dist_f);
		list.push(b, 2.);

		let nodes = [list.pop_lowest_f(), list.pop_lowest_f()];

		assert_eq!([Some(a), Some(b)], nodes);
	}
}
