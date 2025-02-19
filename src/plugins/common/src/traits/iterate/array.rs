use super::Iterate;

impl<TItem, const N: usize> Iterate for [TItem; N] {
	type TItem<'a>
		= &'a TItem
	where
		Self: 'a;

	fn iterate(&self) -> impl Iterator<Item = Self::TItem<'_>> {
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
