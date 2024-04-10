use crate::{
	skill::{Queued, Skill, SkillState},
	traits::{Enqueue, GetStateManager, Iter, IterMut, TryDequeue},
};
use bevy::{ecs::component::Component, utils::default};
use common::traits::state_duration::StateDuration;
use std::{collections::VecDeque, marker::PhantomData, time::Duration};

#[derive(Debug, PartialEq)]
pub struct DequeueAble;

#[derive(Debug, PartialEq)]
pub struct EnqueueAble;

#[derive(Component, Debug, PartialEq)]
pub enum Queue<TEnqueue = QueueCollection<EnqueueAble>, TDequeue = QueueCollection<DequeueAble>> {
	Enqueue(TEnqueue),
	Dequeue(TDequeue),
}

impl Default for Queue {
	fn default() -> Self {
		Queue::Dequeue(QueueCollection {
			queue: default(),
			duration: None,
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
	duration: Option<Duration>,
	state: PhantomData<TState>,
}

impl<T> Clone for QueueCollection<T> {
	fn clone(&self) -> Self {
		Self {
			queue: self.queue.clone(),
			duration: self.duration,
			state: PhantomData,
		}
	}
}

impl From<QueueCollection<EnqueueAble>> for QueueCollection<DequeueAble> {
	fn from(value: QueueCollection<EnqueueAble>) -> Self {
		QueueCollection {
			queue: value.queue,
			duration: value.duration,
			state: PhantomData,
		}
	}
}

impl From<QueueCollection<DequeueAble>> for QueueCollection<EnqueueAble> {
	fn from(value: QueueCollection<DequeueAble>) -> Self {
		QueueCollection {
			queue: value.queue,
			duration: value.duration,
			state: PhantomData,
		}
	}
}

impl<TState> QueueCollection<TState> {
	#[cfg(test)]
	pub(crate) fn new<const N: usize>(queue: [Skill<Queued>; N]) -> Self {
		Self {
			queue: VecDeque::from(queue),
			duration: None,
			state: PhantomData,
		}
	}
}

impl Enqueue<Skill<Queued>> for QueueCollection<EnqueueAble> {
	fn enqueue(&mut self, item: Skill<Queued>) {
		self.queue.push_back(item);
	}
}

impl TryDequeue<Skill<Queued>> for QueueCollection<DequeueAble> {
	fn try_dequeue(&mut self) -> Option<Skill<Queued>> {
		if self.duration.is_some() {
			return None;
		}
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
			(queue.try_dequeue(), queue)
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
			([queue.try_dequeue(), queue.try_dequeue()], queue)
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

	#[test]
	fn clone() {
		let queue = QueueCollection {
			duration: Some(Duration::from_millis(42)),
			queue: VecDeque::from([Skill {
				data: Queued(SlotKey::Hand(Side::Main)),
				..default()
			}]),
			state: PhantomData::<()>,
		};

		assert_eq!(queue, queue.clone())
	}

	#[test]
	fn dequeue_to_enqueue() {
		let queue = QueueCollection {
			duration: Some(Duration::from_millis(42)),
			queue: VecDeque::from([Skill {
				data: Queued(SlotKey::Hand(Side::Main)),
				..default()
			}]),
			state: PhantomData::<DequeueAble>,
		};

		assert_eq!(
			QueueCollection {
				duration: Some(Duration::from_millis(42)),
				queue: VecDeque::from([Skill {
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}]),
				state: PhantomData::<EnqueueAble>,
			},
			queue.into()
		)
	}

	#[test]
	fn enqueue_to_dequeue() {
		let queue = QueueCollection {
			duration: Some(Duration::from_millis(42)),
			queue: VecDeque::from([Skill {
				data: Queued(SlotKey::Hand(Side::Main)),
				..default()
			}]),
			state: PhantomData::<EnqueueAble>,
		};

		assert_eq!(
			QueueCollection {
				duration: Some(Duration::from_millis(42)),
				queue: VecDeque::from([Skill {
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}]),
				state: PhantomData::<DequeueAble>,
			},
			queue.into()
		)
	}
}

impl StateDuration<SkillState> for (&mut Duration, &Skill<Queued>) {
	fn get_state_duration(&self, key: SkillState) -> Duration {
		match key {
			SkillState::Aim => self.1.cast.aim,
			SkillState::PreCast => self.1.cast.pre,
			SkillState::Active => self.1.cast.active,
			SkillState::AfterCast => self.1.cast.after,
		}
	}

	fn elapsed_mut(&mut self) -> &mut Duration {
		self.0
	}
}

impl GetStateManager for QueueCollection<DequeueAble> {
	fn get_state_manager(&mut self) -> Option<impl StateDuration<SkillState>> {
		let skill = self.queue.front()?;

		if self.duration.is_none() {
			self.duration = Some(Duration::default());
		}

		Some((self.duration.as_mut()?, skill))
	}

	fn clear_state_manager(&mut self) {
		self.duration = None;
	}
}

#[cfg(test)]
mod test_state_duration {
	use super::*;
	use crate::{components::SlotKey, skill::Cast};
	use common::components::Side;

	#[test]
	fn get_phasing_times() {
		let mut queue = QueueCollection {
			duration: Some(Duration::default()),
			queue: VecDeque::from([Skill {
				data: Queued(SlotKey::Hand(Side::Main)),
				cast: Cast {
					aim: Duration::from_millis(42),
					pre: Duration::from_millis(1),
					active: Duration::from_millis(2),
					after: Duration::from_millis(3),
				},
				..default()
			}]),
			state: PhantomData,
		};

		let manager = queue.get_state_manager().unwrap();

		assert_eq!(
			[
				(Duration::from_millis(42), SkillState::Aim),
				(Duration::from_millis(1), SkillState::PreCast),
				(Duration::from_millis(2), SkillState::Active),
				(Duration::from_millis(3), SkillState::AfterCast),
			],
			[
				SkillState::Aim,
				SkillState::PreCast,
				SkillState::Active,
				SkillState::AfterCast
			]
			.map(|state| (manager.get_state_duration(state), state))
		)
	}

	#[test]
	fn get_phasing_times_from_first() {
		let mut queue = QueueCollection {
			duration: Some(Duration::default()),
			queue: VecDeque::from([
				Skill {
					data: Queued(SlotKey::Hand(Side::Main)),
					cast: Cast {
						aim: Duration::from_millis(42),
						pre: Duration::from_millis(1),
						active: Duration::from_millis(2),
						after: Duration::from_millis(3),
					},
					..default()
				},
				Skill {
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				},
			]),
			state: PhantomData,
		};

		let manager = queue.get_state_manager().unwrap();

		assert_eq!(
			[
				(Duration::from_millis(42), SkillState::Aim),
				(Duration::from_millis(1), SkillState::PreCast),
				(Duration::from_millis(2), SkillState::Active),
				(Duration::from_millis(3), SkillState::AfterCast),
			],
			[
				SkillState::Aim,
				SkillState::PreCast,
				SkillState::Active,
				SkillState::AfterCast
			]
			.map(|state| (manager.get_state_duration(state), state))
		)
	}

	#[test]
	fn get_duration() {
		let mut queue = QueueCollection {
			duration: Some(Duration::from_secs(11)),
			queue: VecDeque::from([Skill {
				data: Queued(SlotKey::Hand(Side::Main)),
				..default()
			}]),
			state: PhantomData,
		};

		let mut manager = queue.get_state_manager().unwrap();

		assert_eq!(&mut Duration::from_secs(11), manager.elapsed_mut())
	}

	#[test]
	fn clear_duration_when_calling_clear() {
		let mut queue = QueueCollection {
			duration: Some(Duration::from_secs(11)),
			queue: VecDeque::from([Skill {
				data: Queued(SlotKey::Hand(Side::Main)),
				..default()
			}]),
			state: PhantomData,
		};

		queue.clear_state_manager();

		assert_eq!(None, queue.duration);
	}

	#[test]
	fn no_dequeue_when_duration_set() {
		let mut queue = QueueCollection {
			duration: Some(Duration::from_secs(11)),
			queue: VecDeque::from([Skill {
				data: Queued(SlotKey::Hand(Side::Main)),
				..default()
			}]),
			state: PhantomData,
		};

		assert_eq!(
			(
				None,
				VecDeque::from([Skill {
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}])
			),
			(queue.try_dequeue(), queue.queue)
		);
	}

	#[test]
	fn set_default_duration_when_getting_manager() {
		let mut queue = QueueCollection {
			duration: None,
			queue: VecDeque::from([Skill {
				data: Queued(SlotKey::Hand(Side::Main)),
				..default()
			}]),
			state: PhantomData,
		};

		assert_eq!(
			(Some(Duration::default()), Some(Duration::default())),
			(
				queue.get_state_manager().map(|mut m| *m.elapsed_mut()),
				queue.duration
			)
		);
	}

	#[test]
	fn do_not_set_default_duration_when_getting_manager_when_queue_is_empty() {
		let mut queue = QueueCollection {
			duration: None,
			queue: default(),
			state: PhantomData::<DequeueAble>,
		};

		queue.get_state_manager();

		assert_eq!(None, queue.duration);
	}
}
