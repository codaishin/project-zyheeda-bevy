pub(crate) mod walk_back;

use std::{collections::HashMap, hash::Hash};

#[derive(Debug, PartialEq, Default, Clone)]
pub(crate) struct ClosedList<TNode>
where
	TNode: Eq + Hash + Copy,
{
	start: TNode,
	parents: HashMap<TNode, TNode>,
}

impl<TNode> ClosedList<TNode>
where
	TNode: Eq + Hash + Copy,
{
	pub(crate) fn new(start: TNode) -> Self {
		Self {
			start,
			parents: HashMap::from([(start, start)]),
		}
	}

	pub(crate) fn start(&self) -> &TNode {
		&self.start
	}

	pub(crate) fn insert(&mut self, node: TNode, comes_from: TNode) {
		self.parents.insert(node, comes_from);
	}

	pub(crate) fn parent(&self, node: &TNode) -> Option<&TNode> {
		self.parents.get(node)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn start() {
		let start = "start";
		let list = ClosedList::new(start);

		assert_eq!(&start, list.start());
	}

	#[test]
	fn parent() {
		let node = "node";
		let parent = "parent";
		let mut list = ClosedList::new("start");

		list.insert(node, parent);

		assert_eq!(Some(&parent), list.parent(&node));
	}
}
