use index::Index;
use std::{iter::FusedIterator, ops::Deref};

#[derive(Debug, PartialEq)]
pub struct RingBuffer<T, const CAPACITY: usize> {
	items: [Option<T>; CAPACITY],
	head: Index<CAPACITY>,
	tail: Index<CAPACITY>,
	len: usize,
}

impl<T, const CAPACITY: usize> RingBuffer<T, CAPACITY> {
	pub fn push_back(&mut self, item: T) {
		let back = *self.head;

		let Some(head) = self.items.get_mut(back) else {
			return;
		};
		*head = Some(item);

		self.head = self.head.incremented();
		if self.len == CAPACITY {
			self.tail = self.tail.incremented();
		} else {
			self.len += 1;
		}
	}

	pub fn push_front(&mut self, item: T) {
		let front = self.tail.decremented();

		let Some(tail) = self.items.get_mut(*front) else {
			return;
		};

		*tail = Some(item);

		self.tail = front;
		if self.len == CAPACITY {
			self.head = self.head.decremented();
		} else {
			self.len += 1;
		}
	}

	pub fn iter(&self) -> Iter<'_, T, CAPACITY> {
		Iter {
			index: self.tail,
			buffer: self,
			seen: 0,
		}
	}

	pub fn len(&self) -> usize {
		self.len
	}

	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}
}

impl<T, const CAPACITY: usize> Default for RingBuffer<T, CAPACITY> {
	fn default() -> Self {
		Self {
			items: [const { None }; CAPACITY],
			tail: Index::ZERO,
			head: Index::ZERO,
			len: 0,
		}
	}
}

impl<T, const CAPACITY: usize> From<[T; CAPACITY]> for RingBuffer<T, CAPACITY> {
	fn from(items: [T; CAPACITY]) -> Self {
		Self {
			items: items.map(|v| Some(v)),
			tail: Index::ZERO,
			head: Index::ZERO,
			len: CAPACITY,
		}
	}
}

impl<T, const CAPACITY: usize> IntoIterator for RingBuffer<T, CAPACITY> {
	type Item = T;
	type IntoIter = IntoIter<T, CAPACITY>;

	fn into_iter(self) -> Self::IntoIter {
		IntoIter {
			index: self.tail,
			buffer: self,
			seen: 0,
		}
	}
}

pub struct IntoIter<T, const CAPACITY: usize> {
	buffer: RingBuffer<T, CAPACITY>,
	index: Index<CAPACITY>,
	seen: usize,
}

impl<T, const CAPACITY: usize> Iterator for IntoIter<T, CAPACITY> {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		if self.seen == self.buffer.len() {
			return None;
		};

		let next = &mut self.buffer.items[*self.index];

		self.index = self.index.incremented();
		self.seen += 1;

		next.take()
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		let remaining = self.buffer.len() - self.seen;

		(remaining, Some(remaining))
	}
}

impl<T, const CAPACITY: usize> ExactSizeIterator for IntoIter<T, CAPACITY> {}

impl<T, const CAPACITY: usize> FusedIterator for IntoIter<T, CAPACITY> {}

pub struct Iter<'a, T, const CAPACITY: usize> {
	buffer: &'a RingBuffer<T, CAPACITY>,
	index: Index<CAPACITY>,
	seen: usize,
}

impl<'a, T, const CAPACITY: usize> Iterator for Iter<'a, T, CAPACITY> {
	type Item = &'a T;

	fn next(&mut self) -> Option<Self::Item> {
		if self.seen == self.buffer.len() {
			return None;
		};

		let next = self.buffer.items[*self.index].as_ref();

		self.index = self.index.incremented();
		self.seen += 1;

		next
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		let remaining = self.buffer.len() - self.seen;

		(remaining, Some(remaining))
	}
}

impl<'a, T, const CAPACITY: usize> ExactSizeIterator for Iter<'a, T, CAPACITY> {}

impl<'a, T, const CAPACITY: usize> FusedIterator for Iter<'a, T, CAPACITY> {}

mod index {
	use super::*;

	#[derive(Debug, PartialEq, Default, Clone, Copy)]
	pub(super) struct Index<const CAPACITY: usize>(usize);

	impl<const CAPACITY: usize> Index<CAPACITY> {
		const MAX: Self = Self(CAPACITY.saturating_sub(1));

		pub(super) const ZERO: Self = Self(0);

		#[must_use]
		pub(super) fn incremented(self) -> Self {
			if self == Self::MAX {
				return Self::ZERO;
			}

			Self(*self + 1)
		}

		#[must_use]
		pub(super) fn decremented(self) -> Self {
			if self == Self::ZERO {
				return Self::MAX;
			}

			Self(*self - 1)
		}
	}

	impl<const CAPACITY: usize> Deref for Index<CAPACITY> {
		type Target = usize;

		fn deref(&self) -> &Self::Target {
			&self.0
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	mod into_iter {
		use super::*;

		#[test]
		fn instantiate_empty() {
			let buff = RingBuffer::<f32, 11>::default();

			assert_eq!(vec![] as Vec<f32>, buff.into_iter().collect::<Vec<_>>());
		}

		#[test]
		fn instantiate_from_array() {
			let buff = RingBuffer::from([1, 2, 3, 4, 5]);

			assert_eq!(vec![1, 2, 3, 4, 5], buff.into_iter().collect::<Vec<_>>());
		}

		#[test]
		fn push_back_item_on_full_buffer() {
			let mut buff = RingBuffer::from([1, 2, 3, 4, 5]);

			buff.push_back(6);

			assert_eq!(vec![2, 3, 4, 5, 6], buff.into_iter().collect::<Vec<_>>());
		}

		#[test]
		fn push_back_item_on_full_buffer_multiple_times() {
			let mut buff = RingBuffer::from([1, 2, 3, 4, 5]);

			buff.push_back(6);
			buff.push_back(7);
			buff.push_back(8);

			assert_eq!(vec![4, 5, 6, 7, 8], buff.into_iter().collect::<Vec<_>>());
		}

		#[test]
		fn push_back_item_on_full_buffer_multiple_times_with_overflow() {
			let mut buff = RingBuffer::from([1, 2, 3, 4, 5]);

			buff.push_back(6);
			buff.push_back(7);
			buff.push_back(8);
			buff.push_back(9);
			buff.push_back(10);
			buff.push_back(11);

			assert_eq!(vec![7, 8, 9, 10, 11], buff.into_iter().collect::<Vec<_>>());
		}

		#[test]
		fn push_back_not_full() {
			let mut buff = RingBuffer::<i32, 100>::default();

			buff.push_back(1);
			buff.push_back(2);
			buff.push_back(3);

			assert_eq!(vec![1, 2, 3], buff.into_iter().collect::<Vec<_>>());
		}

		#[test]
		fn push_front_item_on_full_buffer() {
			let mut buff = RingBuffer::from([1, 2, 3, 4, 5]);

			buff.push_front(6);

			assert_eq!(vec![6, 1, 2, 3, 4], buff.into_iter().collect::<Vec<_>>());
		}

		#[test]
		fn push_front_item_on_full_buffer_multiple_times() {
			let mut buff = RingBuffer::from([1, 2, 3, 4, 5]);

			buff.push_front(6);
			buff.push_front(7);
			buff.push_front(8);

			assert_eq!(vec![8, 7, 6, 1, 2], buff.into_iter().collect::<Vec<_>>());
		}

		#[test]
		fn push_front_item_on_full_buffer_multiple_times_with_overflow() {
			let mut buff = RingBuffer::from([1, 2, 3, 4, 5]);

			buff.push_front(6);
			buff.push_front(7);
			buff.push_front(8);
			buff.push_front(9);
			buff.push_front(10);
			buff.push_front(11);

			assert_eq!(vec![11, 10, 9, 8, 7], buff.into_iter().collect::<Vec<_>>());
		}

		#[test]
		fn push_front_not_full() {
			let mut buff = RingBuffer::<i32, 100>::default();

			buff.push_front(1);
			buff.push_front(2);
			buff.push_front(3);

			assert_eq!(vec![3, 2, 1], buff.into_iter().collect::<Vec<_>>());
		}

		#[test]
		fn push_back_and_front_alternating_over_capacity() {
			let mut buff = RingBuffer::<i32, 3>::default();

			buff.push_front(1);
			buff.push_back(2);
			buff.push_front(3);
			buff.push_back(4);

			assert_eq!(vec![1, 2, 4], buff.into_iter().collect::<Vec<_>>());
		}

		#[test]
		fn size_hint() {
			let mut buff = RingBuffer::<i32, 5>::default();
			buff.push_back(1);
			buff.push_front(2);
			buff.push_back(3);
			let mut it = buff.into_iter();

			let hints = [
				it.size_hint(),
				{
					it.next();
					it.size_hint()
				},
				{
					it.next();
					it.size_hint()
				},
				{
					it.next();
					it.size_hint()
				},
				{
					it.next();
					it.size_hint()
				},
			];

			assert_eq!(
				[
					(3, Some(3)),
					(2, Some(2)),
					(1, Some(1)),
					(0, Some(0)),
					(0, Some(0))
				],
				hints
			);
		}
	}

	mod len {
		use super::*;

		#[test]
		fn empty() {
			let buff = RingBuffer::<i32, 5>::default();

			assert_eq!(0, buff.len());
		}

		#[test]
		fn partially_filled() {
			let mut buff = RingBuffer::<i32, 5>::default();

			buff.push_back(1);
			buff.push_front(2);
			buff.push_back(3);

			assert_eq!(3, buff.len());
		}

		#[test]
		fn full() {
			let mut buff = RingBuffer::<i32, 5>::default();

			buff.push_back(1);
			buff.push_front(2);
			buff.push_back(3);
			buff.push_front(4);
			buff.push_back(5);

			assert_eq!(5, buff.len());
		}

		#[test]
		fn from_array() {
			let buff = RingBuffer::<i32, 5>::from([1, 2, 3, 4, 5]);

			assert_eq!(5, buff.len());
		}
	}

	mod iter {
		use super::*;

		#[test]
		fn empty() {
			let buff = RingBuffer::<i32, 5>::default();

			assert_eq!(vec![] as Vec<&i32>, buff.iter().collect::<Vec<_>>());
		}

		#[test]
		fn partially_filled() {
			let mut buff = RingBuffer::<i32, 5>::default();

			buff.push_back(1);
			buff.push_front(2);
			buff.push_back(3);

			assert_eq!(vec![&2, &1, &3], buff.iter().collect::<Vec<_>>());
		}

		#[test]
		fn full() {
			let mut buff = RingBuffer::<i32, 5>::default();

			buff.push_back(1);
			buff.push_front(2);
			buff.push_back(3);
			buff.push_front(4);
			buff.push_back(5);

			assert_eq!(vec![&4, &2, &1, &3, &5], buff.iter().collect::<Vec<_>>());
		}

		#[test]
		fn from_array() {
			let buff = RingBuffer::<i32, 5>::from([1, 2, 3, 4, 5]);

			assert_eq!(vec![&1, &2, &3, &4, &5], buff.iter().collect::<Vec<_>>());
		}

		#[test]
		fn size_hint() {
			let mut buff = RingBuffer::<i32, 5>::default();
			buff.push_back(1);
			buff.push_front(2);
			buff.push_back(3);
			let mut it = buff.iter();

			let hints = [
				it.size_hint(),
				{
					it.next();
					it.size_hint()
				},
				{
					it.next();
					it.size_hint()
				},
				{
					it.next();
					it.size_hint()
				},
				{
					it.next();
					it.size_hint()
				},
			];

			assert_eq!(
				[
					(3, Some(3)),
					(2, Some(2)),
					(1, Some(1)),
					(0, Some(0)),
					(0, Some(0))
				],
				hints
			);
		}
	}

	mod zero_capacity {
		use super::*;

		#[test]
		fn default_into_iter() {
			let buff = RingBuffer::<i32, 0>::default();

			assert_eq!(vec![] as Vec<i32>, buff.into_iter().collect::<Vec<_>>());
		}

		#[test]
		fn default_iter() {
			let buff = RingBuffer::<i32, 0>::default();

			assert_eq!(vec![] as Vec<&i32>, buff.iter().collect::<Vec<_>>());
		}

		#[test]
		fn push_back() {
			let mut buff = RingBuffer::<i32, 0>::default();

			buff.push_back(1);

			assert_eq!(vec![] as Vec<i32>, buff.into_iter().collect::<Vec<_>>());
		}

		#[test]
		fn push_front() {
			let mut buff = RingBuffer::<i32, 0>::default();

			buff.push_front(1);

			assert_eq!(vec![] as Vec<i32>, buff.into_iter().collect::<Vec<_>>());
		}
	}
}
