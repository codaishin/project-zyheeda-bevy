use super::{nav_grid_node::NavGridNode, path_iterator::PathIterator};
use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub(crate) struct ClosedList {
	start: NavGridNode,
	parents: HashMap<NavGridNode, NavGridNode>,
}

impl ClosedList {
	pub(crate) fn new(start: NavGridNode) -> Self {
		Self {
			start,
			parents: HashMap::from([(start, start)]),
		}
	}

	pub(crate) fn start(&self) -> &NavGridNode {
		&self.start
	}

	pub(crate) fn insert(&mut self, node: NavGridNode, comes_from: NavGridNode) {
		self.parents.insert(node, comes_from);
	}

	pub(crate) fn iter_back_from(self, node: NavGridNode) -> PathIterator {
		PathIterator::new(self, node)
	}

	pub(crate) fn parent(&self, node: &NavGridNode) -> Option<&NavGridNode> {
		self.parents.get(node)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn start() {
		let start = NavGridNode { x: 1, y: 2 };
		let list = ClosedList::new(start);

		assert_eq!(&start, list.start());
	}

	#[test]
	fn parent() {
		let node = NavGridNode { x: 1, y: 2 };
		let parent = NavGridNode { x: 3, y: 4 };
		let mut list = ClosedList::new(NavGridNode::default());

		list.insert(node, parent);

		assert_eq!(Some(&parent), list.parent(&node));
	}

	#[test]
	fn iterate() {
		let a = NavGridNode { x: 1, y: 2 };
		let b = NavGridNode { x: 2, y: 2 };
		let c = NavGridNode { x: 3, y: 2 };
		let mut list = ClosedList::new(a);
		list.insert(b, a);
		list.insert(c, b);

		let path = list.iter_back_from(c).collect::<Vec<_>>();

		assert_eq!(vec![c, b, a], path);
	}
}
