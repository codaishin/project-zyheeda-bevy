use super::ClosedList;
use crate::tools::nav_grid_node::NavGridNode;

#[derive(Debug, Clone)]
pub(crate) struct WalkBack {
	list: ClosedList,
	next: Option<NavGridNode>,
}

impl ClosedList {
	pub(crate) fn walk_back_from(self, node: NavGridNode) -> WalkBack {
		WalkBack {
			list: self,
			next: Some(node),
		}
	}
}

impl WalkBack {
	fn parent(&self, node: &NavGridNode) -> Option<&NavGridNode> {
		if node == self.list.start() {
			return None;
		}

		self.list.parent(node)
	}
}

impl Iterator for WalkBack {
	type Item = NavGridNode;

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
		let a = NavGridNode { x: 1, y: 2 };
		let b = NavGridNode { x: 2, y: 2 };
		let c = NavGridNode { x: 3, y: 2 };
		let mut list = ClosedList::new(a);
		list.insert(b, a);
		list.insert(c, b);
		let path = list.walk_back_from(c);

		let nodes = path.collect::<Vec<_>>();

		assert_eq!(vec![c, b, a], nodes);
	}
}
