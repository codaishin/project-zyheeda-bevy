use super::Iterate;

impl<TItem> Iterate<TItem> for [TItem] {
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
		let slc = [1, 2, 3];

		assert_eq!(
			slc.as_slice().iter().collect::<Vec<_>>(),
			slc.as_slice().iterate().collect::<Vec<_>>(),
		);
	}
}
