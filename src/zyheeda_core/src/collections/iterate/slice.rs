use super::Iterate;
use std::slice::Iter;

impl<'a, TItem> Iterate<'a> for [TItem]
where
	Self: 'a,
{
	type TItem = &'a TItem;
	type TIter = Iter<'a, TItem>;

	fn iterate(&'a self) -> Self::TIter {
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
