use super::Iterate;

impl<TItem, const N: usize> Iterate<TItem> for [TItem; N] {
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
		let arr = [1, 2, 3];

		assert_eq!(
			arr.iter().collect::<Vec<_>>(),
			arr.iterate().collect::<Vec<_>>(),
		);
	}
}
