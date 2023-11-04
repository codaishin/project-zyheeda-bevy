use bevy::prelude::Component;
use std::collections::{vec_deque::Iter, VecDeque};

#[derive(Component)]
pub struct Queue<T>
where
	T: Copy,
{
	queue: VecDeque<T>,
	popped_last: Option<T>,
}

impl<T> Queue<T>
where
	T: Copy,
{
	pub fn new<const N: usize>(values: [T; N]) -> Self {
		Self {
			queue: values.into(),
			popped_last: None,
		}
	}

	pub fn pop_front(&mut self) -> Option<T> {
		let front = self.queue.pop_front();
		self.popped_last = front;

		front
	}

	pub fn popped_last(&self) -> Option<T> {
		self.popped_last
	}

	pub fn push_back(&mut self, value: T) {
		self.queue.push_back(value)
	}

	pub fn clear(&mut self) {
		self.queue.clear()
	}

	pub fn len(&self) -> usize {
		self.queue.len()
	}

	pub fn is_empty(&self) -> bool {
		self.queue.is_empty()
	}

	pub fn iter(&self) -> Iter<T> {
		self.queue.iter()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn iter() {
		let queue = Queue::new([1, 2, 3]);
		let queue = queue.iter().cloned().collect::<Vec<u32>>();

		assert_eq!(vec![1, 2, 3], queue);
	}

	#[test]
	fn is_empty_false() {
		let queue = Queue::new([1, 2, 3]);

		assert!(!queue.is_empty());
	}

	#[test]
	fn is_empty_true() {
		let queue = Queue::<u32>::new([]);

		assert!(queue.is_empty());
	}

	#[test]
	fn len_3() {
		let queue = Queue::new([1, 2, 3]);

		assert_eq!(3, queue.len());
	}

	#[test]
	fn len_5() {
		let queue = Queue::new([1, 2, 3, 2, 1]);

		assert_eq!(5, queue.len());
	}

	#[test]
	fn clear() {
		let mut queue = Queue::new([1, 2, 3]);
		queue.clear();

		assert_eq!(0, queue.len());
	}

	#[test]
	fn push_back() {
		let mut queue = Queue::new([1, 2, 3]);
		queue.push_back(42);
		let queue: Vec<u32> = queue.iter().cloned().collect();

		assert_eq!(vec![1, 2, 3, 42], queue);
	}

	#[test]
	fn pop_front() {
		let mut queue = Queue::new([1, 2, 3]);
		let popped = queue.pop_front();
		let queue: Vec<u32> = queue.iter().cloned().collect();

		assert_eq!((Some(1), vec![2, 3]), (popped, queue));
	}

	#[test]
	fn popped_last_none_when_never_popped() {
		let queue = Queue::new([1, 2, 3]);

		assert_eq!(None, queue.popped_last());
	}

	#[test]
	fn popped_last_some() {
		let mut queue = Queue::new([1, 2, 3]);
		_ = queue.pop_front();

		assert_eq!(Some(1), queue.popped_last());
	}
}
