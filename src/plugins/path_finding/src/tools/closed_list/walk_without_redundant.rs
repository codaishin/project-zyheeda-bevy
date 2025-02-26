use crate::tools::nav_grid_node::NavGridNode;

pub(crate) struct WalkWithoutRedundant<TLos, TIterator> {
	los: TLos,
	iterator: TIterator,
	next_override: Option<NavGridNode>,
}

pub(crate) trait WithoutRedundantNodes: Iterator<Item = NavGridNode> + Sized {
	fn without_redundant_nodes<TLos>(self, los: TLos) -> WalkWithoutRedundant<TLos, Self>
	where
		TLos: Fn(NavGridNode, NavGridNode) -> bool;
}

impl<TIterator> WithoutRedundantNodes for TIterator
where
	Self: Iterator<Item = NavGridNode> + Sized,
{
	fn without_redundant_nodes<TLos>(self, los: TLos) -> WalkWithoutRedundant<TLos, Self>
	where
		TLos: Fn(NavGridNode, NavGridNode) -> bool,
	{
		WalkWithoutRedundant {
			los,
			next_override: None,
			iterator: self,
		}
	}
}

impl<TLos, TIterator> Iterator for WalkWithoutRedundant<TLos, TIterator>
where
	TLos: Fn(NavGridNode, NavGridNode) -> bool,
	TIterator: Iterator<Item = NavGridNode>,
{
	type Item = NavGridNode;

	fn next(&mut self) -> Option<Self::Item> {
		if let Some(next) = self.next_override.take() {
			return Some(next);
		}

		let current = self.iterator.next()?;

		for next in self.iterator.by_ref() {
			self.next_override = Some(next);
			if !(self.los)(current, next) {
				break;
			}
		}

		Some(current)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use mockall::{automock, predicate::eq};

	#[automock]
	trait _LosF {
		fn call(&self, a: NavGridNode, b: NavGridNode) -> bool;
	}

	macro_rules! new_los_f {
		($setup:expr) => {{
			let mut mock = Mock_LosF::default();
			$setup(&mut mock);
			move |a, b| mock.call(a, b)
		}};
	}

	#[test]
	fn iter_all_nodes() {
		let a = NavGridNode { x: 1, y: 2 };
		let b = NavGridNode { x: 2, y: 2 };
		let c = NavGridNode { x: 3, y: 2 };
		let iter = vec![a, b, c].into_iter();
		let los = new_los_f!(|mock: &mut Mock_LosF| {
			mock.expect_call().return_const(false);
		});

		let nodes = iter.without_redundant_nodes(los).collect::<Vec<_>>();

		assert_eq!(vec![a, b, c], nodes);
	}

	#[test]
	fn skip_redundant_node() {
		let a = NavGridNode { x: 1, y: 2 };
		let b = NavGridNode { x: 2, y: 2 };
		let c = NavGridNode { x: 3, y: 2 };
		let iter = vec![a, b, c].into_iter();
		let los = new_los_f!(|mock: &mut Mock_LosF| {
			mock.expect_call().with(eq(a), eq(b)).return_const(true);
			mock.expect_call().with(eq(a), eq(c)).return_const(true);
			mock.expect_call().return_const(false);
		});

		let nodes = iter.without_redundant_nodes(los).collect::<Vec<_>>();

		assert_eq!(vec![a, c], nodes);
	}
}
