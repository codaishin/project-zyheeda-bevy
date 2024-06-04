use super::Iterate;

impl<TItem> Iterate<TItem> for Vec<TItem> {
	fn iterate<'a>(&'a self) -> impl DoubleEndedIterator<Item = &'a TItem>
	where
		TItem: 'a,
	{
		self.iter()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn iter_through_iter_trait() {
		let vec = vec![1, 2, 3];

		assert_eq!(
			vec.iter().collect::<Vec<_>>(),
			vec.iterate().collect::<Vec<_>>(),
		);
	}
}
