use crate::{
	skill::{Queued, Skill},
	traits::{Dequeue, Enqueue, Iter, IterMut},
};
use bevy::{ecs::component::Component, utils::default};
use std::{collections::VecDeque, marker::PhantomData};

#[derive(Debug, PartialEq)]
pub struct DequeueAble;

pub struct EnqueueAble;

#[derive(Component, PartialEq, Debug)]
pub struct Queue<TState = DequeueAble> {
	queue: VecDeque<Skill<Queued>>,
	state: PhantomData<TState>,
}

impl Default for Queue {
	fn default() -> Self {
		Self {
			queue: default(),
			state: default(),
		}
	}
}

impl Queue {
	#[cfg(test)]
	fn new<const N: usize>(queue: [Skill<Queued>; N]) -> Self {
		Self {
			queue: VecDeque::from(queue),
			state: PhantomData,
		}
	}

	pub fn to_enqueue_able(self) -> Queue<EnqueueAble> {
		Queue {
			queue: self.queue,
			state: PhantomData,
		}
	}
}

impl Queue<EnqueueAble> {
	pub fn to_dequeue_able(self) -> Queue<DequeueAble> {
		Queue {
			queue: self.queue,
			state: PhantomData,
		}
	}
}

impl Enqueue<Skill<Queued>> for Queue<EnqueueAble> {
	fn enqueue(&mut self, item: Skill<Queued>) {
		self.queue.push_back(item);
	}
}

impl Dequeue<Skill<Queued>> for Queue {
	fn dequeue(&mut self) -> Option<Skill<Queued>> {
		self.queue.pop_front()
	}
}

impl<TState> Iter<Skill<Queued>> for Queue<TState> {
	fn iter<'a>(&'a self) -> impl DoubleEndedIterator<Item = &'a Skill<Queued>>
	where
		Skill<Queued>: 'a,
	{
		self.queue.iter()
	}
}

impl IterMut<Skill<Queued>> for Queue<EnqueueAble> {
	fn iter_mut<'a>(&'a mut self) -> impl DoubleEndedIterator<Item = &'a mut Skill<Queued>>
	where
		Skill<Queued>: 'a,
	{
		self.queue.iter_mut()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::SlotKey;
	use bevy::utils::default;
	use common::components::Side;

	#[test]
	fn enqueue_one_skill() {
		let mut queue = Queue::default().to_enqueue_able();
		queue.enqueue(Skill {
			name: "my skill",
			data: Queued(SlotKey::Hand(Side::Main)),
			..default()
		});

		assert_eq!(
			Queue::new([Skill {
				name: "my skill",
				data: Queued(SlotKey::Hand(Side::Main)),
				..default()
			}]),
			queue.to_dequeue_able()
		);
	}

	#[test]
	fn enqueue_two_skills() {
		let mut queue = Queue::default().to_enqueue_able();
		queue.enqueue(Skill {
			name: "skill a",
			data: Queued(SlotKey::Hand(Side::Off)),
			..default()
		});
		queue.enqueue(Skill {
			name: "skill b",
			data: Queued(SlotKey::Hand(Side::Main)),
			..default()
		});

		assert_eq!(
			Queue::new([
				Skill {
					name: "skill a",
					data: Queued(SlotKey::Hand(Side::Off)),
					..default()
				},
				Skill {
					name: "skill b",
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}
			]),
			queue.to_dequeue_able()
		);
	}

	#[test]
	fn dequeue_one_skill() {
		let mut queue = Queue::new([Skill {
			name: "my skill",
			data: Queued(SlotKey::Hand(Side::Main)),
			..default()
		}]);

		assert_eq!(
			(
				Some(Skill {
					name: "my skill",
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}),
				Queue::default()
			),
			(queue.dequeue(), queue)
		);
	}

	#[test]
	fn dequeue_two_skill() {
		let mut queue = Queue::new([
			Skill {
				name: "skill a",
				data: Queued(SlotKey::Hand(Side::Off)),
				..default()
			},
			Skill {
				name: "skill b",
				data: Queued(SlotKey::Hand(Side::Main)),
				..default()
			},
		]);

		assert_eq!(
			(
				[
					Some(Skill {
						name: "skill a",
						data: Queued(SlotKey::Hand(Side::Off)),
						..default()
					}),
					Some(Skill {
						name: "skill b",
						data: Queued(SlotKey::Hand(Side::Main)),
						..default()
					})
				],
				Queue::default()
			),
			([queue.dequeue(), queue.dequeue()], queue)
		);
	}

	#[test]
	fn as_slice() {
		let mut queue = Queue::default().to_enqueue_able();
		queue.enqueue(Skill {
			name: "skill a",
			data: Queued(SlotKey::Hand(Side::Off)),
			..default()
		});
		queue.enqueue(Skill {
			name: "skill b",
			data: Queued(SlotKey::Hand(Side::Main)),
			..default()
		});

		assert_eq!(
			vec![
				&Skill {
					name: "skill a",
					data: Queued(SlotKey::Hand(Side::Off)),
					..default()
				},
				&Skill {
					name: "skill b",
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}
			],
			queue.iter().collect::<Vec<_>>()
		)
	}

	#[test]
	fn as_slice_mut() {
		let mut queue = Queue::default().to_enqueue_able();
		queue.enqueue(Skill {
			name: "skill a",
			data: Queued(SlotKey::Hand(Side::Off)),
			..default()
		});
		queue.enqueue(Skill {
			name: "skill b",
			data: Queued(SlotKey::Hand(Side::Main)),
			..default()
		});

		assert_eq!(
			vec![
				&Skill {
					name: "skill a",
					data: Queued(SlotKey::Hand(Side::Off)),
					..default()
				},
				&Skill {
					name: "skill b",
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}
			],
			queue.iter_mut().collect::<Vec<_>>()
		)
	}
}
