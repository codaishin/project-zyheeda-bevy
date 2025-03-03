pub(crate) struct WalkWithoutRedundant<TLos, TIterator>
where
	TIterator: Iterator,
{
	los: TLos,
	iterator: TIterator,
	next_override: Option<TIterator::Item>,
}

pub(crate) trait WithoutRedundantNodes: Iterator + Sized {
	fn without_redundant_nodes<TLos>(self, los: TLos) -> WalkWithoutRedundant<TLos, Self>
	where
		TLos: Fn(&Self::Item, &Self::Item) -> bool;
}

impl<TIterator> WithoutRedundantNodes for TIterator
where
	Self: Iterator + Sized,
{
	fn without_redundant_nodes<TLos>(self, los: TLos) -> WalkWithoutRedundant<TLos, Self>
	where
		TLos: Fn(&Self::Item, &Self::Item) -> bool,
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
	TLos: Fn(&TIterator::Item, &TIterator::Item) -> bool,
	TIterator: Iterator,
	TIterator::Item: Copy,
{
	type Item = TIterator::Item;

	fn next(&mut self) -> Option<Self::Item> {
		if let Some(next) = self.next_override.take() {
			return Some(next);
		}

		let current = self.iterator.next()?;

		for next in self.iterator.by_ref() {
			self.next_override = Some(next);
			if !(self.los)(&current, &next) {
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
		fn call(&self, a: &u8, b: &u8) -> bool;
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
		let a = 1;
		let b = 2;
		let c = 3;
		let iter = vec![a, b, c].into_iter();

		let nodes = iter
			.without_redundant_nodes(new_los_f!(|mock: &mut Mock_LosF| {
				mock.expect_call().return_const(false);
			}))
			.collect::<Vec<_>>();

		assert_eq!(vec![a, b, c], nodes);
	}

	#[test]
	fn skip_redundant_node() {
		let a = 1;
		let b = 2;
		let c = 3;
		let iter = vec![a, b, c].into_iter();

		let nodes = iter
			.without_redundant_nodes(new_los_f!(|mock: &mut Mock_LosF| {
				mock.expect_call().with(eq(a), eq(b)).return_const(true);
				mock.expect_call().with(eq(a), eq(c)).return_const(true);
				mock.expect_call().return_const(false);
			}))
			.collect::<Vec<_>>();

		assert_eq!(vec![a, c], nodes);
	}
}
