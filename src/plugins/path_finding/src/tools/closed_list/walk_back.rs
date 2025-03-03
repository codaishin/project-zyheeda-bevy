use super::ClosedList;
use std::hash::Hash;

#[derive(Debug, Clone)]
pub(crate) struct WalkBack<TNode>
where
	TNode: Eq + Hash + Copy,
{
	list: ClosedList<TNode>,
	next: Option<TNode>,
}

impl<TNode> ClosedList<TNode>
where
	TNode: Eq + Hash + Copy,
{
	/// Creates an iterator of [`NavGridNode`]s representing a path
	/// in reversed order.
	///
	/// <div class="warning">
	///   If there is no parent for the node stored in the closed list,
	///   the iterator will only contain this one node.
	/// </div>
	pub(crate) fn walk_back_from(self, node: TNode) -> WalkBack<TNode> {
		WalkBack {
			list: self,
			next: Some(node),
		}
	}
}

impl<TNode> WalkBack<TNode>
where
	TNode: Eq + Hash + Copy,
{
	fn parent(&self, node: &TNode) -> Option<&TNode> {
		if node == self.list.start() {
			return None;
		}

		self.list.parent(node)
	}
}

impl<TNode> Iterator for WalkBack<TNode>
where
	TNode: Eq + Hash + Copy,
{
	type Item = TNode;

	fn next(&mut self) -> Option<Self::Item> {
		let current = self.next?;

		self.next = self.parent(&current).copied();

		Some(current)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn iterate_backwards() {
		let a = 1;
		let b = 2;
		let c = 3;
		let mut list = ClosedList::new(a);
		list.insert(b, a);
		list.insert(c, b);
		let path = list.walk_back_from(c);

		let nodes = path.collect::<Vec<_>>();

		assert_eq!(vec![c, b, a], nodes);
	}
}
