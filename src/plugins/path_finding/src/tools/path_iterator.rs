use super::{closed_list::ClosedList, nav_grid_node::NavGridNode};

#[derive(Debug, Clone)]
pub(crate) struct PathIterator {
	list: ClosedList,
	next: Option<NavGridNode>,
}

impl PathIterator {
	pub(crate) fn new(list: ClosedList, end: NavGridNode) -> Self {
		Self {
			list,
			next: Some(end),
		}
	}

	fn parent(&self, node: &NavGridNode) -> Option<&NavGridNode> {
		if node == &self.list.start {
			return None;
		}

		self.list.parent(node)
	}

	pub(crate) fn remove_redundant_nodes<T>(self, line_of_sight: T) -> CleanedPathIterator<T>
	where
		T: Fn(NavGridNode, NavGridNode) -> bool + Clone,
	{
		CleanedPathIterator {
			los: line_of_sight,
			iterator: self,
		}
	}
}

impl Iterator for PathIterator {
	type Item = NavGridNode;

	fn next(&mut self) -> Option<Self::Item> {
		let current = self.next?;

		self.next = self.parent(&current).copied();

		Some(current)
	}
}

#[derive(Debug, Clone)]
pub struct CleanedPathIterator<T>
where
	T: Fn(NavGridNode, NavGridNode) -> bool + Clone,
{
	los: T,
	iterator: PathIterator,
}

impl<T> CleanedPathIterator<T>
where
	T: Fn(NavGridNode, NavGridNode) -> bool + Clone,
{
	fn try_move_closer_to(
		node: &mut NavGridNode,
		target: &NavGridNode,
		other_los_node: &NavGridNode,
		los: &impl Fn(NavGridNode, NavGridNode) -> bool,
	) {
		let Some(direction) = node.eight_sided_direction_to(target) else {
			return;
		};

		loop {
			let moved = *node + direction;

			if &moved == target {
				return;
			}
			if !los(moved, *target) {
				return;
			}
			if !los(moved, *other_los_node) {
				return;
			}

			*node = moved;
		}
	}

	fn try_override_nodes(
		has_line_of_sight: &impl Fn(NavGridNode, NavGridNode) -> bool,
		node: &NavGridNode,
		last: &NavGridNode,
		next: &NavGridNode,
	) -> Vec<NavGridNode> {
		let Some(dir_last) = node.eight_sided_direction_to(last) else {
			return vec![*node];
		};
		let Some(dir_next) = node.eight_sided_direction_to(next) else {
			return vec![*node];
		};
		if dir_last.is_diagonal() && dir_next.is_diagonal() {
			return vec![*node];
		}
		if dir_last.is_straight() && dir_next.is_straight() {
			return vec![*node];
		}

		let mut override_nodes = (*node, *node);
		let do_override = |a, b, (old_a, old_b): (NavGridNode, NavGridNode)| {
			has_line_of_sight(a, b) && (a - b).right_angle_len() > (old_a - old_b).right_angle_len()
		};

		let mut to_last = *node + dir_last;

		while &to_last != last {
			let mut to_next = *node + dir_next;

			while &to_next != next {
				if do_override(to_last, to_next, override_nodes) {
					override_nodes = (to_last, to_next);
				} else {
					break;
				}
				to_next += dir_next;
			}
			to_last += dir_last;
		}

		if override_nodes != (*node, *node) {
			vec![override_nodes.0, override_nodes.1]
		} else {
			vec![*node]
		}
	}

	pub(crate) fn collect_with_optimized_node_positions<TResult>(self) -> Vec<TResult>
	where
		TResult: From<NavGridNode>,
	{
		let los = &self.los.clone();

		let first_pass = &self.collect::<Vec<_>>();
		let second_pass = first_pass
			.iter()
			.enumerate()
			.map(move |(i, node)| {
				let mut node = *node;

				if i == 0 {
					return node;
				}
				let Some(last) = first_pass.get(i - 1) else {
					return node;
				};
				let Some(next) = first_pass.get(i + 1) else {
					return node;
				};

				Self::try_move_closer_to(&mut node, last, next, los);
				Self::try_move_closer_to(&mut node, next, last, los);

				node
			})
			.collect::<Vec<_>>();

		second_pass
			.iter()
			.enumerate()
			.flat_map(|(i, node)| {
				if i == 0 {
					return vec![*node];
				}
				let Some(last) = second_pass.get(i - 1) else {
					return vec![*node];
				};
				let Some(next) = second_pass.get(i + 1) else {
					return vec![*node];
				};

				Self::try_override_nodes(los, node, last, next)
			})
			.map(TResult::from)
			.collect()
	}
}

impl<T> Iterator for CleanedPathIterator<T>
where
	T: Fn(NavGridNode, NavGridNode) -> bool + Clone,
{
	type Item = NavGridNode;

	fn next(&mut self) -> Option<Self::Item> {
		let current = self.iterator.next?;

		self.iterator.next = match self.iterator.parent(&current).copied() {
			None => None,
			Some(mut explored) => {
				let mut last_visible = explored;

				while let Some(parent) = self.iterator.parent(&explored) {
					if (self.los)(current, *parent) {
						last_visible = *parent;
					}

					explored = *parent;
				}

				Some(last_visible)
			}
		};

		Some(current)
	}
}
