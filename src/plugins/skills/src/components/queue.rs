use crate::{
	skill::{Queued, Skill},
	traits::{Dequeue, Enqueue, Iter, IterMut},
};
use bevy::ecs::component::Component;
use std::collections::VecDeque;

#[derive(Component, PartialEq, Debug, Default)]
pub struct Queue(pub VecDeque<Skill<Queued>>);

impl Enqueue<Skill<Queued>> for Queue {
	fn enqueue(&mut self, item: Skill<Queued>) {
		self.0.push_back(item);
	}
}

impl Dequeue<Skill<Queued>> for Queue {
	fn dequeue(&mut self) -> Option<Skill<Queued>> {
		self.0.pop_front()
	}
}

impl Iter<Skill<Queued>> for Queue {
	fn iter<'a>(&'a self) -> impl DoubleEndedIterator<Item = &'a Skill<Queued>>
	where
		Skill<Queued>: 'a,
	{
		self.0.iter()
	}
}

impl IterMut<Skill<Queued>> for Queue {
	fn iter_mut<'a>(&'a mut self) -> impl DoubleEndedIterator<Item = &'a mut Skill<Queued>>
	where
		Skill<Queued>: 'a,
	{
		self.0.iter_mut()
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
		let mut queue = Queue::default();
		queue.enqueue(Skill {
			name: "my skill",
			data: Queued(SlotKey::Hand(Side::Main)),
			..default()
		});

		assert_eq!(
			Queue(VecDeque::from([Skill {
				name: "my skill",
				data: Queued(SlotKey::Hand(Side::Main)),
				..default()
			}])),
			queue
		);
	}

	#[test]
	fn enqueue_two_skills() {
		let mut queue = Queue::default();
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
			Queue(VecDeque::from([
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
			])),
			queue
		);
	}

	#[test]
	fn dequeue_one_skill() {
		let mut queue = Queue(VecDeque::from([Skill {
			name: "my skill",
			data: Queued(SlotKey::Hand(Side::Main)),
			..default()
		}]));

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
		let mut queue = Queue(VecDeque::from([
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
		let mut queue = Queue::default();
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
		let mut queue = Queue::default();
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
