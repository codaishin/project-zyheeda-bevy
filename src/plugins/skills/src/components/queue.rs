use crate::{
	skill::{Queued, Skill},
	traits::{Dequeue, Enqueue, Iter, IterMut},
};
use bevy::{ecs::component::Component, utils::default};
use std::{collections::VecDeque, marker::PhantomData};

#[derive(Debug, PartialEq)]
pub struct DequeueAble;

#[derive(Debug, PartialEq)]
pub struct EnqueueAble;

#[derive(Component)]
pub enum Queue<TEnqueue = QueueCollection<EnqueueAble>, TDequeue = QueueCollection<DequeueAble>> {
	Enqueue(TEnqueue),
	Dequeue(TDequeue),
}

impl Default for Queue {
	fn default() -> Self {
		Queue::Dequeue(QueueCollection {
			queue: default(),
			state: PhantomData,
		})
	}
}

impl Iter<Skill<Queued>> for Queue {
	fn iter<'a>(&'a self) -> impl DoubleEndedIterator<Item = &'a Skill<Queued>>
	where
		Skill<Queued>: 'a,
	{
		match self {
			Queue::Dequeue(q) => q.queue.iter(),
			Queue::Enqueue(q) => q.queue.iter(),
		}
	}
}

#[cfg(test)]
mod test_queue {
	use super::*;
	use crate::components::SlotKey;
	use common::components::Side;

	#[test]
	fn iter_dequeue_able() {
		let queue = Queue::Dequeue(QueueCollection::new([
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
		]));

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
	fn iter_enqueue_able() {
		let queue = Queue::Enqueue(QueueCollection::new([
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
		]));

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
}

#[derive(PartialEq, Debug)]
pub struct QueueCollection<TState> {
	queue: VecDeque<Skill<Queued>>,
	state: PhantomData<TState>,
}

impl<TState> QueueCollection<TState> {
	#[cfg(test)]
	pub(crate) fn new<const N: usize>(queue: [Skill<Queued>; N]) -> Self {
		Self {
			queue: VecDeque::from(queue),
			state: PhantomData,
		}
	}
}

impl Enqueue<Skill<Queued>> for QueueCollection<EnqueueAble> {
	fn enqueue(&mut self, item: Skill<Queued>) {
		self.queue.push_back(item);
	}
}

impl Dequeue<Skill<Queued>> for QueueCollection<DequeueAble> {
	fn dequeue(&mut self) -> Option<Skill<Queued>> {
		self.queue.pop_front()
	}
}

impl IterMut<Skill<Queued>> for QueueCollection<EnqueueAble> {
	fn iter_mut<'a>(&'a mut self) -> impl DoubleEndedIterator<Item = &'a mut Skill<Queued>>
	where
		Skill<Queued>: 'a,
	{
		self.queue.iter_mut()
	}
}

#[cfg(test)]
mod test_queue_collection {
	use super::*;
	use crate::components::SlotKey;
	use bevy::utils::default;
	use common::components::Side;

	#[test]
	fn enqueue_one_skill() {
		let mut queue = QueueCollection::new([]);
		queue.enqueue(Skill {
			name: "my skill",
			data: Queued(SlotKey::Hand(Side::Main)),
			..default()
		});

		assert_eq!(
			QueueCollection::new([Skill {
				name: "my skill",
				data: Queued(SlotKey::Hand(Side::Main)),
				..default()
			}]),
			queue
		);
	}

	#[test]
	fn enqueue_two_skills() {
		let mut queue = QueueCollection::new([]);
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
			QueueCollection::new([
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
			]),
			queue
		);
	}

	#[test]
	fn dequeue_one_skill() {
		let mut queue = QueueCollection::new([Skill {
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
				QueueCollection::new([])
			),
			(queue.dequeue(), queue)
		);
	}

	#[test]
	fn dequeue_two_skill() {
		let mut queue = QueueCollection::new([
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
				QueueCollection::new([])
			),
			([queue.dequeue(), queue.dequeue()], queue)
		);
	}

	#[test]
	fn as_slice_mut() {
		let mut queue = QueueCollection::new([]);
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
