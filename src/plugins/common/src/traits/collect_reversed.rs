mod vec;

pub trait CollectReversed: Iterator {
	fn collect_reversed<TCollection>(self) -> TCollection
	where
		TCollection: FromIterator<Self::Item> + ReversAble;
}

pub trait ReversAble {
	fn reverse_collection(&mut self);
}

impl<TIterator> CollectReversed for TIterator
where
	Self: Iterator,
{
	fn collect_reversed<TCollection>(self) -> TCollection
	where
		TCollection: FromIterator<Self::Item> + ReversAble,
	{
		let mut c = TCollection::from_iter(self);
		c.reverse_collection();

		c
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn collect_reversed_into_vector() {
		let iter = vec![1, 2, 3].into_iter();

		assert_eq!(vec![3, 2, 1], iter.collect_reversed::<Vec<_>>());
	}
}
