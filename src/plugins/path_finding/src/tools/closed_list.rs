use super::{nav_grid_node::NavGridNode, path_iterator::PathIterator};
use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub(crate) struct ClosedList {
	pub(crate) start: NavGridNode,
	parents: HashMap<NavGridNode, NavGridNode>,
}

impl ClosedList {
	pub(crate) fn new(start: NavGridNode) -> Self {
		Self {
			start,
			parents: HashMap::from([(start, start)]),
		}
	}

	pub(crate) fn insert(&mut self, node: NavGridNode, comes_from: NavGridNode) {
		self.parents.insert(node, comes_from);
	}

	pub(crate) fn construct_path_from(self, node: NavGridNode) -> PathIterator {
		PathIterator::new(self, node)
	}

	pub(crate) fn parent(&self, node: &NavGridNode) -> Option<&NavGridNode> {
		self.parents.get(node)
	}
}
