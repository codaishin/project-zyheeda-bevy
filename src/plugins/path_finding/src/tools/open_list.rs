use std::{
	cmp::{Ordering, Reverse},
	collections::BinaryHeap,
};

pub(crate) struct OpenList<TNode, TDistFn>
where
	TNode: Eq,
	TDistFn: Fn(&TNode, &TNode) -> f32,
{
	end: TNode,
	heap: BinaryHeap<Reverse<Node<TNode>>>,
	dist_f: TDistFn,
}

impl<TNode, TDistFn> OpenList<TNode, TDistFn>
where
	TNode: Eq + Copy,
	TDistFn: Fn(&TNode, &TNode) -> f32,
{
	pub(crate) fn new(start: TNode, end: TNode, dist_f: TDistFn) -> Self {
		let heap = BinaryHeap::from([Reverse(Node {
			node: start,
			f: dist_f(&start, &end),
		})]);
		OpenList { end, dist_f, heap }
	}

	pub(crate) fn pop_lowest_f(&mut self) -> Option<TNode> {
		self.heap.pop().map(|Reverse(Node { node, .. })| node)
	}

	pub(crate) fn push(&mut self, node: TNode, g: f32) {
		let f = g + self.dist(&node, &self.end);
		self.heap.push(Reverse(Node { node, f }));
	}

	fn dist(&self, a: &TNode, b: &TNode) -> f32 {
		(self.dist_f)(a, b)
	}
}

#[derive(Debug, PartialEq)]
struct Node<TNode>
where
	TNode: Eq,
{
	node: TNode,
	f: f32,
}

impl<TNode> Eq for Node<TNode> where TNode: Eq {}

impl<TNode> PartialOrd for Node<TNode>
where
	TNode: Eq,
{
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl<TNode> Ord for Node<TNode>
where
	TNode: Eq,
{
	fn cmp(&self, other: &Self) -> Ordering {
		let Some(c_f) = self.f.partial_cmp(&other.f) else {
			panic!(
				"tried to compare {} with {} (f values are not comparable)",
				self.f, other.f
			);
		};

		c_f
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use mockall::{automock, predicate::eq};

	#[automock]
	trait _DistF {
		fn call(&self, a: &'static str, b: &'static str) -> f32;
	}

	macro_rules! new_dist_f {
		($setup:expr) => {{
			let mut mock = Mock_DistF::default();
			$setup(&mut mock);
			move |&a, &b| mock.call(a, b)
		}};
	}

	#[test]
	fn new() {
		let start = "start";
		let end = "end";
		let mut list = OpenList::new(
			start,
			end,
			new_dist_f!(|mock: &mut Mock_DistF| {
				mock.expect_call()
					.times(1)
					.with(eq(start), eq(end))
					.return_const(42.);
			}),
		);

		let node = list.pop_lowest_f();

		assert_eq!(Some(start), node);
	}

	#[test]
	fn pop_by_lowest_f_value() {
		let a = "a";
		let b = "b";
		let end = "end";
		let mut list = OpenList::new(
			a,
			end,
			new_dist_f!(|mock: &mut Mock_DistF| {
				mock.expect_call().with(eq(a), eq(end)).return_const(42.);
				mock.expect_call().with(eq(b), eq(end)).return_const(11.);
			}),
		);
		list.push(b, 0.);

		let nodes = [list.pop_lowest_f(), list.pop_lowest_f()];

		assert_eq!([Some(b), Some(a)], nodes);
	}

	#[test]
	fn pop_by_lowest_f_value_combined_with_g() {
		let a = "a";
		let b = "b";
		let end = "end";
		let mut list = OpenList::new(
			a,
			end,
			new_dist_f!(|mock: &mut Mock_DistF| {
				mock.expect_call().with(eq(a), eq(end)).return_const(12.);
				mock.expect_call().with(eq(b), eq(end)).return_const(11.);
			}),
		);
		list.push(b, 2.);

		let nodes = [list.pop_lowest_f(), list.pop_lowest_f()];

		assert_eq!([Some(a), Some(b)], nodes);
	}
}
