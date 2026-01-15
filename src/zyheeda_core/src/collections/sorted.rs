use std::{marker::PhantomData, ops::Deref, vec::IntoIter};

#[derive(Debug, PartialEq, Clone)]
pub struct Sorted<T, TContainer = Vec<T>>
where
	T: Ord,
{
	data: TContainer,
	_p: PhantomData<fn() -> T>,
}

impl<T> Sorted<T>
where
	T: Ord,
{
	pub fn as_slice(&self) -> &[T] {
		&self.data
	}
}

impl<T, TContainer> Sorted<T, TContainer>
where
	T: Ord,
	TContainer: internal::UpdateSortedContainer<T>,
{
	pub fn push(&mut self, item: T) {
		let index = self.data.order_preserving_insertion_index(&item);
		self.data.insert(index, item);
	}
}

impl<T, TContainer> Default for Sorted<T, TContainer>
where
	T: Ord,
	TContainer: internal::Empty,
{
	fn default() -> Self {
		Self {
			data: TContainer::empty(),
			_p: PhantomData,
		}
	}
}

impl<T, TContainer, const N: usize> From<[T; N]> for Sorted<T, TContainer>
where
	T: Ord,
	TContainer: internal::NewSortedContainer<T>,
{
	fn from(source: [T; N]) -> Self {
		Self::from_iter(source)
	}
}

impl<T, TContainer> FromIterator<T> for Sorted<T, TContainer>
where
	T: Ord,
	TContainer: internal::NewSortedContainer<T>,
{
	fn from_iter<TSource>(source: TSource) -> Self
	where
		TSource: IntoIterator<Item = T>,
	{
		Self {
			data: TContainer::new_sorted(source),
			_p: PhantomData,
		}
	}
}

impl<T> Deref for Sorted<T>
where
	T: Ord,
{
	type Target = [T];

	fn deref(&self) -> &Self::Target {
		&self.data
	}
}

impl<T> IntoIterator for Sorted<T>
where
	T: Ord,
{
	type Item = T;
	type IntoIter = IntoIter<T>;

	fn into_iter(self) -> Self::IntoIter {
		self.data.into_iter()
	}
}

mod internal {
	pub type OrderPreservingInsertionIndex = usize;

	pub trait Empty {
		fn empty() -> Self;
	}

	pub trait NewSortedContainer<T> {
		fn new_sorted<TItems>(items: TItems) -> Self
		where
			TItems: IntoIterator<Item = T>;
	}

	pub trait UpdateSortedContainer<T>: Empty {
		fn order_preserving_insertion_index(&self, item: &T) -> OrderPreservingInsertionIndex;
		fn insert(&mut self, index: OrderPreservingInsertionIndex, item: T);
	}
	impl<T> Empty for Vec<T> {
		fn empty() -> Self {
			Self::new()
		}
	}

	impl<T> UpdateSortedContainer<T> for Vec<T>
	where
		T: Ord,
	{
		fn order_preserving_insertion_index(&self, item: &T) -> OrderPreservingInsertionIndex {
			match self.binary_search(item) {
				Ok(index) | Err(index) => index,
			}
		}

		fn insert(&mut self, index: usize, item: T) {
			self.insert(index, item);
		}
	}

	impl<T> NewSortedContainer<T> for Vec<T>
	where
		T: Ord,
	{
		fn new_sorted<TItems>(items: TItems) -> Self
		where
			TItems: IntoIterator<Item = T>,
		{
			let mut items = Self::from_iter(items);
			items.sort();
			items
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use test_case::test_case;

	#[test]
	fn deref_empty() {
		let sorted = Sorted::<i32>::default();

		let sliced = &*sorted;

		assert_eq!((&[]) as &[i32], sliced);
	}

	#[test]
	fn from_iterator() {
		let sorted = Sorted::from_iter(std::iter::repeat_n(4, 10));

		assert_eq!(&[4; 10], &*sorted);
	}

	#[test]
	fn from_array() {
		let sorted = Sorted::from([1, 2, 3]);

		assert_eq!(&[1, 2, 3], &*sorted);
	}

	#[test]
	fn from_unsorted() {
		let sorted = Sorted::from([3, 2, 1]);

		assert_eq!(&[1, 2, 3], &*sorted);
	}

	#[test]
	fn as_slice() {
		let sorted = Sorted::from([1, 2, 3]);

		let sliced = sorted.as_slice();

		assert_eq!(&[1, 2, 3], sliced);
	}

	#[test_case([1]; "item 1")]
	#[test_case([2, 1]; "items 2 1")]
	#[test_case([1, 3, 2]; "items 1 3 2")]
	#[test_case([-21, 30, 2, -11, 2, 200, 89, -7, -6]; "many items")]
	fn push<const N: usize>(mut items: [i32; N]) {
		let mut sorted = Sorted::default();

		for item in items {
			sorted.push(item)
		}

		items.sort();
		assert_eq!(items.as_slice(), sorted.as_slice());
	}

	#[test]
	fn into_iter() {
		let sorted = Sorted::from([1, 2, 3]);

		let iter = sorted.into_iter();

		assert_eq!(vec![1, 2, 3], iter.collect::<Vec<_>>());
	}

	mod container_usage_default {
		use super::*;
		use mockall::mock;

		mock! {
			_Container {}
			impl internal::Empty for _Container {
				fn empty() -> Self;
			}
		}

		#[test]
		fn default() {
			let ctx = Mock_Container::empty_context();
			ctx.expect().times(1).returning(Mock_Container::default);

			_ = Sorted::<i32, Mock_Container>::default();
		}
	}

	mod container_usage_push {
		use super::*;
		use mockall::{mock, predicate::eq};

		mock! {
			_Container {}
			impl internal::Empty for _Container {
				fn empty() -> Self;
			}
			impl internal::UpdateSortedContainer<i32> for _Container {
				fn order_preserving_insertion_index(&self, item: &i32) -> internal::OrderPreservingInsertionIndex;
				fn insert(&mut self, index: internal::OrderPreservingInsertionIndex, item: i32);
			}
		}

		#[test]
		fn push_item() {
			let ctx = Mock_Container::empty_context();
			ctx.expect().times(1).returning(|| {
				let mut mock = Mock_Container::default();
				mock.expect_order_preserving_insertion_index()
					.with(eq(1))
					.times(1)
					.return_const(42_usize);
				mock.expect_insert()
					.with(eq(42), eq(1))
					.once()
					.return_const(());
				mock
			});

			let mut sorted = Sorted::<i32, Mock_Container>::default();

			sorted.push(1);
		}
	}

	mod container_usage_bulk {
		use super::*;

		struct _Container {
			items: Vec<i32>,
		}

		impl internal::NewSortedContainer<i32> for _Container {
			fn new_sorted<TItems>(items: TItems) -> Self
			where
				TItems: IntoIterator<Item = i32>,
			{
				Self {
					items: items.into_iter().collect(),
				}
			}
		}

		#[test]
		fn from_array() {
			let sorted = Sorted::<i32, _Container>::from([1, 2, 3]);

			assert_eq!(vec![1, 2, 3], sorted.data.items);
		}

		#[test]
		fn from_iter() {
			let sorted = Sorted::<i32, _Container>::from_iter([1, 2, 3]);

			assert_eq!(vec![1, 2, 3], sorted.data.items);
		}
	}
}
