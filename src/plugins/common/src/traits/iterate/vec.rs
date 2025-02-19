use super::Iterate;

impl<TItem> Iterate for Vec<TItem> {
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
		let vec = vec![1, 2, 3];

		assert_eq!(
			vec.iter().collect::<Vec<_>>(),
			vec.iterate().collect::<Vec<_>>(),
		);
	}
}
