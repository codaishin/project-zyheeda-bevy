use super::nav_grid_node::NavGridNode;
use std::{collections::HashMap, marker::PhantomData};

#[derive(Debug, PartialEq, Default, Clone)]
pub(crate) struct ClosedList<TIterator> {
	start: NavGridNode,
	parents: HashMap<NavGridNode, NavGridNode>,
	_i: PhantomData<TIterator>,
}

impl<TIterator> ClosedList<TIterator> {
	pub(crate) fn new(start: NavGridNode) -> Self {
		Self {
			_i: PhantomData,
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

impl<TIterator> ClosedList<TIterator>
where
	TIterator: From<(Self, NavGridNode)>,
{
	pub(crate) fn iter_back_from(self, node: NavGridNode) -> TIterator {
		TIterator::from((self, node))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug, PartialEq)]
	struct _Iter(ClosedList<_Iter>, NavGridNode);

	impl From<(ClosedList<_Iter>, NavGridNode)> for _Iter {
		fn from((list, node): (ClosedList<_Iter>, NavGridNode)) -> Self {
			Self(list, node)
		}
	}

	#[test]
	fn start() {
		let start = NavGridNode { x: 1, y: 2 };
		let list = ClosedList::<_Iter>::new(start);

		assert_eq!(&start, list.start());
	}

	#[test]
	fn parent() {
		let node = NavGridNode { x: 1, y: 2 };
		let parent = NavGridNode { x: 3, y: 4 };
		let mut list = ClosedList::<_Iter>::new(NavGridNode::default());

		list.insert(node, parent);

		assert_eq!(Some(&parent), list.parent(&node));
	}

	#[test]
	fn iterate() {
		let node = NavGridNode { x: 1, y: 2 };
		let start = NavGridNode { x: 5, y: 11 };
		let list = ClosedList::<_Iter>::new(start);

		let iter = list.iter_back_from(node);

		assert_eq!(_Iter(ClosedList::<_Iter>::new(start), node), iter);
	}
}
