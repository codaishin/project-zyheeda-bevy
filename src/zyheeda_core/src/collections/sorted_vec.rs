use std::{ops::Deref, vec::IntoIter};

#[derive(Debug, PartialEq, Clone)]
pub struct SortedVec<T>
where
	T: Ord,
{
	data: Vec<T>,
}

impl<T> SortedVec<T>
where
	T: Ord,
{
	pub fn as_slice(&self) -> &[T] {
		&self.data
	}

	pub fn push(&mut self, item: T) {
		self.data.push(item);
		self.data.sort();
	}
}

impl<T> Default for SortedVec<T>
where
	T: Ord,
{
	fn default() -> Self {
		Self { data: vec![] }
	}
}

impl<T, const N: usize> From<[T; N]> for SortedVec<T>
where
	T: Ord,
{
	fn from(source: [T; N]) -> Self {
		Self::from_iter(source)
	}
}

impl<T> FromIterator<T> for SortedVec<T>
where
	T: Ord,
{
	fn from_iter<TSource>(source: TSource) -> Self
	where
		TSource: IntoIterator<Item = T>,
	{
		let mut data = source.into_iter().collect::<Vec<_>>();

		data.sort();

		Self { data }
	}
}

impl<T> Deref for SortedVec<T>
where
	T: Ord,
{
	type Target = [T];

	fn deref(&self) -> &Self::Target {
		&self.data
	}
}

impl<T> IntoIterator for SortedVec<T>
where
	T: Ord,
{
	type Item = T;
	type IntoIter = IntoIter<T>;

	fn into_iter(self) -> Self::IntoIter {
		self.data.into_iter()
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use std::collections::HashSet;
	use test_case::test_case;

	#[test]
	fn deref_empty() {
		let sorted = SortedVec::<i32>::default();

		let sliced = &*sorted;

		assert_eq!((&[]) as &[i32], sliced);
	}

	#[test]
	fn from_iterator() {
		let sorted = SortedVec::from_iter(std::iter::repeat_n(4, 10));

		assert_eq!(&[4; 10], &*sorted);
	}

	#[test]
	fn from_array() {
		let sorted = SortedVec::from([1, 2, 3]);

		_ = HashSet::from([1, 2, 3]);

		assert_eq!(&[1, 2, 3], &*sorted);
	}

	#[test]
	fn from_unsorted() {
		let sorted = SortedVec::from([3, 2, 1]);

		assert_eq!(&[1, 2, 3], &*sorted);
	}

	#[test]
	fn as_slice() {
		let sorted = SortedVec::from([1, 2, 3]);

		let sliced = sorted.as_slice();

		assert_eq!(&[1, 2, 3], sliced);
	}

	#[test_case([1]; "item 1")]
	#[test_case([2, 1]; "items 2 1")]
	#[test_case([1, 3, 2]; "items 1 3 2")]
	fn push<const N: usize>(mut items: [i32; N]) {
		let mut sorted = SortedVec::default();

		for item in items {
			sorted.push(item)
		}

		items.sort();
		assert_eq!(items.as_slice(), sorted.as_slice());
	}

	#[test]
	fn into_iter() {
		let sorted = SortedVec::from([1, 2, 3]);

		let iter = sorted.into_iter();

		assert_eq!(vec![1, 2, 3], iter.collect::<Vec<_>>());
	}
}
