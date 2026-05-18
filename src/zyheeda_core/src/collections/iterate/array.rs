use super::Iterate;
use std::slice::Iter;

impl<'a, TItem, const N: usize> Iterate<'a> for [TItem; N]
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
		let arr = [1, 2, 3];

		assert_eq!(
			arr.iter().collect::<Vec<_>>(),
			arr.iterate().collect::<Vec<_>>(),
		);
	}
}
