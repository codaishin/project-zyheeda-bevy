pub(crate) mod walk_back;
pub(crate) mod walk_without_redundant;

use super::nav_grid_node::NavGridNode;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Default, Clone)]
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
}
