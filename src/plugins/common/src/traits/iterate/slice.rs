use super::Iterate;

impl<TItem> Iterate for [TItem] {
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
		let slc = [1, 2, 3];

		assert_eq!(
			slc.as_slice().iter().collect::<Vec<_>>(),
			slc.as_slice().iterate().collect::<Vec<_>>(),
		);
	}
}
