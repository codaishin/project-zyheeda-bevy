use super::SlotKey;
use crate::{
	skill::{PlayerSkills, Queued, Skill, SkillState, StartBehaviorFn, StopBehaviorFn},
	traits::{
		Enqueue,
		Execution,
		GetActiveSkill,
		GetAnimation,
		GetOldLastMut,
		GetSlots,
		Iter,
		IterMut,
		IterRecentMut,
		TryDequeue,
	},
};
use bevy::{ecs::component::Component, utils::default};
use common::{
	components::{Animate, Side},
	traits::state_duration::StateDuration,
};
use std::{collections::VecDeque, time::Duration};

#[derive(Debug, PartialEq, Clone)]
pub struct DequeueAble;

impl From<usize> for DequeueAble {
	fn from(_: usize) -> Self {
		Self
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct EnqueueAble {
	start_index: usize,
}

impl From<usize> for EnqueueAble {
	fn from(start_index: usize) -> Self {
		Self { start_index }
	}
}

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
			state: DequeueAble,
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
				data: Queued {
					slot_key: SlotKey::Hand(Side::Off),
					..default()
				},
				..default()
			},
			Skill {
				name: "skill b",
				data: Queued {
					slot_key: SlotKey::Hand(Side::Main),
					..default()
				},
				..default()
			},
		]));

		assert_eq!(
			vec![
				&Skill {
					name: "skill a",
					data: Queued {
						slot_key: SlotKey::Hand(Side::Off),
						..default()
					},
					..default()
				},
				&Skill {
					name: "skill b",
					data: Queued {
						slot_key: SlotKey::Hand(Side::Main),
						..default()
					},
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
				data: Queued {
					slot_key: SlotKey::Hand(Side::Off),
					..default()
				},
				..default()
			},
			Skill {
				name: "skill b",
				data: Queued {
					slot_key: SlotKey::Hand(Side::Main),
					..default()
				},
				..default()
			},
		]));

		assert_eq!(
			vec![
				&Skill {
					name: "skill a",
					data: Queued {
						slot_key: SlotKey::Hand(Side::Off),
						..default()
					},
					..default()
				},
				&Skill {
					name: "skill b",
					data: Queued {
						slot_key: SlotKey::Hand(Side::Main),
						..default()
					},
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
	state: TState,
}

impl<TState: Clone> Clone for QueueCollection<TState> {
	fn clone(&self) -> Self {
		Self {
			queue: self.queue.clone(),
			duration: self.duration,
			state: self.state.clone(),
		}
	}
}

impl From<QueueCollection<EnqueueAble>> for QueueCollection<DequeueAble> {
	fn from(value: QueueCollection<EnqueueAble>) -> Self {
		QueueCollection {
			queue: value.queue,
			duration: value.duration,
			state: DequeueAble,
		}
	}
}

impl From<QueueCollection<DequeueAble>> for QueueCollection<EnqueueAble> {
	fn from(value: QueueCollection<DequeueAble>) -> Self {
		let start_index = value.queue.len();

		QueueCollection {
			queue: value.queue,
			duration: value.duration,
			state: EnqueueAble { start_index },
		}
	}
}

impl<TState: From<usize>> QueueCollection<TState> {
	#[cfg(test)]
	pub(crate) fn new<const N: usize>(queue: [Skill<Queued>; N]) -> Self {
		Self {
			queue: VecDeque::from(queue),
			duration: None,
			state: TState::from(N),
		}
	}
}

impl Enqueue<Skill<Queued>> for QueueCollection<EnqueueAble> {
	fn enqueue(&mut self, item: Skill<Queued>) {
		self.queue.push_back(item);
	}
}

impl TryDequeue<Skill<Queued>> for QueueCollection<DequeueAble> {
	fn try_dequeue(&mut self) {
		if self.duration.is_some() {
			return;
		}
		self.queue.pop_front();
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

impl IterRecentMut<Skill<Queued>> for QueueCollection<EnqueueAble> {
	fn iter_recent_mut<'a>(&'a mut self) -> impl DoubleEndedIterator<Item = &'a mut Skill<Queued>>
	where
		Skill<Queued>: 'a,
	{
		self.queue.iter_mut().skip(self.state.start_index)
	}
}

impl GetOldLastMut<Skill<Queued>> for QueueCollection<EnqueueAble> {
	fn get_old_last_mut<'a>(&'a mut self) -> Option<&'a mut Skill<Queued>>
	where
		Skill<Queued>: 'a,
	{
		self.queue.iter_mut().take(self.state.start_index).last()
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
			data: Queued {
				slot_key: SlotKey::Hand(Side::Main),
				..default()
			},
			..default()
		});

		assert_eq!(
			QueueCollection {
				queue: VecDeque::from([Skill {
					name: "my skill",
					data: Queued {
						slot_key: SlotKey::Hand(Side::Main),
						..default()
					},
					..default()
				}]),
				duration: None,
				state: EnqueueAble { start_index: 0 }
			},
			queue
		);
	}

	#[test]
	fn enqueue_two_skills() {
		let mut queue = QueueCollection::new([]);
		queue.enqueue(Skill {
			name: "skill a",
			data: Queued {
				slot_key: SlotKey::Hand(Side::Off),
				..default()
			},
			..default()
		});
		queue.enqueue(Skill {
			name: "skill b",
			data: Queued {
				slot_key: SlotKey::Hand(Side::Main),
				..default()
			},
			..default()
		});

		assert_eq!(
			QueueCollection {
				queue: VecDeque::from([
					Skill {
						name: "skill a",
						data: Queued {
							slot_key: SlotKey::Hand(Side::Off),
							..default()
						},
						..default()
					},
					Skill {
						name: "skill b",
						data: Queued {
							slot_key: SlotKey::Hand(Side::Main),
							..default()
						},
						..default()
					},
				]),
				duration: None,
				state: EnqueueAble { start_index: 0 }
			},
			queue
		);
	}

	#[test]
	fn dequeue_one_skill() {
		let mut queue = QueueCollection::new([Skill {
			name: "my skill",
			data: Queued {
				slot_key: SlotKey::Hand(Side::Main),
				..default()
			},
			..default()
		}]);

		queue.try_dequeue();

		assert_eq!(QueueCollection::new([]), queue);
	}

	#[test]
	fn dequeue_two_skill() {
		let mut queue = QueueCollection::new([
			Skill {
				name: "skill a",
				data: Queued {
					slot_key: SlotKey::Hand(Side::Off),
					..default()
				},
				..default()
			},
			Skill {
				name: "skill b",
				data: Queued {
					slot_key: SlotKey::Hand(Side::Main),
					..default()
				},
				..default()
			},
		]);

		queue.try_dequeue();

		let queue_a: Vec<_> = queue.queue.iter().cloned().collect();

		queue.try_dequeue();

		let queue_b: Vec<_> = queue.queue.iter().cloned().collect();

		assert_eq!(
			(
				vec![Skill {
					name: "skill b",
					data: Queued {
						slot_key: SlotKey::Hand(Side::Main),
						..default()
					},
					..default()
				}],
				vec![]
			),
			(queue_a, queue_b)
		);
	}

	#[test]
	fn as_slice_mut() {
		let mut queue = QueueCollection::new([]);
		queue.enqueue(Skill {
			name: "skill a",
			data: Queued {
				slot_key: SlotKey::Hand(Side::Off),
				..default()
			},
			..default()
		});
		queue.enqueue(Skill {
			name: "skill b",
			data: Queued {
				slot_key: SlotKey::Hand(Side::Main),
				..default()
			},
			..default()
		});

		assert_eq!(
			vec![
				&Skill {
					name: "skill a",
					data: Queued {
						slot_key: SlotKey::Hand(Side::Off),
						..default()
					},
					..default()
				},
				&Skill {
					name: "skill b",
					data: Queued {
						slot_key: SlotKey::Hand(Side::Main),
						..default()
					},
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
				data: Queued {
					slot_key: SlotKey::Hand(Side::Main),
					..default()
				},
				..default()
			}]),
			state: (),
		};

		assert_eq!(queue, queue.clone())
	}

	#[test]
	fn dequeue_to_enqueue() {
		let queue = QueueCollection {
			duration: Some(Duration::from_millis(42)),
			queue: VecDeque::from([Skill {
				data: Queued {
					slot_key: SlotKey::Hand(Side::Main),
					..default()
				},
				..default()
			}]),
			state: DequeueAble,
		};

		assert_eq!(
			QueueCollection {
				duration: Some(Duration::from_millis(42)),
				queue: VecDeque::from([Skill {
					data: Queued {
						slot_key: SlotKey::Hand(Side::Main),
						..default()
					},
					..default()
				}]),
				state: EnqueueAble { start_index: 1 },
			},
			queue.into()
		)
	}

	#[test]
	fn enqueue_to_dequeue() {
		let queue = QueueCollection {
			duration: Some(Duration::from_millis(42)),
			queue: VecDeque::from([Skill {
				data: Queued {
					slot_key: SlotKey::Hand(Side::Main),
					..default()
				},
				..default()
			}]),
			state: EnqueueAble { start_index: 1 },
		};

		assert_eq!(
			QueueCollection {
				duration: Some(Duration::from_millis(42)),
				queue: VecDeque::from([Skill {
					data: Queued {
						slot_key: SlotKey::Hand(Side::Main),
						..default()
					},
					..default()
				}]),
				state: DequeueAble,
			},
			queue.into()
		)
	}

	#[test]
	fn iter_recent_mut() {
		let mut queue = QueueCollection::<EnqueueAble>::new([]);
		queue.enqueue(Skill {
			name: "a",
			data: Queued {
				slot_key: SlotKey::Hand(Side::Main),
				..default()
			},
			..default()
		});
		queue.enqueue(Skill {
			name: "b",
			data: Queued {
				slot_key: SlotKey::Hand(Side::Off),
				..default()
			},
			..default()
		});

		assert_eq!(
			vec![
				&mut Skill {
					name: "a",
					data: Queued {
						slot_key: SlotKey::Hand(Side::Main),
						..default()
					},
					..default()
				},
				&mut Skill {
					name: "b",
					data: Queued {
						slot_key: SlotKey::Hand(Side::Off),
						..default()
					},
					..default()
				}
			],
			queue.iter_recent_mut().collect::<Vec<_>>()
		)
	}

	#[test]
	fn iter_recent_mut_only_new() {
		let mut queue = QueueCollection::<EnqueueAble>::new([Skill {
			name: "a",
			data: Queued {
				slot_key: SlotKey::SkillSpawn,
				..default()
			},
			..default()
		}]);
		queue.enqueue(Skill {
			name: "b",
			data: Queued {
				slot_key: SlotKey::Hand(Side::Main),
				..default()
			},
			..default()
		});
		queue.enqueue(Skill {
			name: "c",
			data: Queued {
				slot_key: SlotKey::Hand(Side::Off),
				..default()
			},
			..default()
		});

		assert_eq!(
			vec![
				&mut Skill {
					name: "b",
					data: Queued {
						slot_key: SlotKey::Hand(Side::Main),
						..default()
					},
					..default()
				},
				&mut Skill {
					name: "c",
					data: Queued {
						slot_key: SlotKey::Hand(Side::Off),
						..default()
					},
					..default()
				}
			],
			queue.iter_recent_mut().collect::<Vec<_>>()
		)
	}

	#[test]
	fn get_old_last_mut() {
		let mut queue = QueueCollection::<EnqueueAble>::new([Skill {
			name: "a",
			data: Queued {
				slot_key: SlotKey::SkillSpawn,
				..default()
			},
			..default()
		}]);

		assert_eq!(
			Some(&mut Skill {
				name: "a",
				data: Queued {
					slot_key: SlotKey::SkillSpawn,
					..default()
				},
				..default()
			}),
			queue.get_old_last_mut()
		)
	}

	#[test]
	fn get_old_last_mut_with_later_enqueues() {
		let mut queue = QueueCollection::<EnqueueAble>::new([Skill {
			name: "a",
			data: Queued {
				slot_key: SlotKey::SkillSpawn,
				..default()
			},
			..default()
		}]);
		queue.enqueue(Skill {
			name: "b",
			data: Queued {
				slot_key: SlotKey::Hand(Side::Main),
				..default()
			},
			..default()
		});

		assert_eq!(
			Some(&mut Skill {
				name: "a",
				data: Queued {
					slot_key: SlotKey::SkillSpawn,
					..default()
				},
				..default()
			}),
			queue.get_old_last_mut()
		)
	}
}

struct ActiveSkill<'a> {
	duration: &'a mut Duration,
	skill: &'a Skill<Queued>,
}

impl GetActiveSkill<PlayerSkills<Side>, SkillState> for QueueCollection<DequeueAble> {
	fn get_active(
		&mut self,
	) -> Option<
		impl Execution + GetAnimation<PlayerSkills<Side>> + GetSlots + StateDuration<SkillState>,
	> {
		let skill = self.queue.front()?;

		if self.duration.is_none() {
			self.duration = Some(Duration::default());
		}

		Some(ActiveSkill {
			duration: self.duration.as_mut()?,
			skill,
		})
	}

	fn clear_active(&mut self) {
		self.duration = None;
	}
}

impl<'a> StateDuration<SkillState> for ActiveSkill<'a> {
	fn get_state_duration(&self, key: SkillState) -> Duration {
		match key {
			SkillState::Aim => self.skill.cast.aim,
			SkillState::PreCast => self.skill.cast.pre,
			SkillState::Active => self.skill.cast.active,
			SkillState::AfterCast => self.skill.cast.after,
		}
	}

	fn elapsed_mut(&mut self) -> &mut Duration {
		self.duration
	}
}

impl<'a> Execution for ActiveSkill<'a> {
	fn get_start(&self) -> Option<StartBehaviorFn> {
		self.skill.execution.run_fn
	}

	fn get_stop(&self) -> Option<StopBehaviorFn> {
		self.skill.execution.stop_fn
	}
}

impl<'a> GetAnimation<PlayerSkills<Side>> for ActiveSkill<'a> {
	fn animate(&self) -> Animate<PlayerSkills<Side>> {
		let Some(animate) = self.skill.animate else {
			return Animate::None;
		};
		match (animate, self.skill.data.slot_key) {
			(PlayerSkills::Shoot(dual_or_single), SlotKey::Hand(side)) => {
				Animate::Repeat(PlayerSkills::Shoot(dual_or_single.on(side)))
			}
			(PlayerSkills::SwordStrike(_), SlotKey::Hand(side)) => {
				Animate::Replay(PlayerSkills::SwordStrike(side))
			}
			_ => Animate::None,
		}
	}
}

impl<'a> GetSlots for ActiveSkill<'a> {
	fn slots(&self) -> Vec<SlotKey> {
		match (self.skill.data.slot_key, self.skill.dual_wield) {
			(SlotKey::Hand(Side::Main), true) => {
				vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)]
			}
			(SlotKey::Hand(Side::Off), true) => {
				vec![SlotKey::Hand(Side::Off), SlotKey::Hand(Side::Main)]
			}
			(slot_key, ..) => vec![slot_key],
		}
	}
}

#[cfg(test)]
mod test_queue_active_skill {
	use super::*;
	use crate::{
		components::{Handed, SideUnset, SlotKey},
		skill::{Cast, SkillExecution, Spawner, Target},
	};
	use bevy::{ecs::system::EntityCommands, transform::components::Transform};
	use common::components::Side;

	#[test]
	fn get_phasing_times() {
		let mut queue = QueueCollection {
			duration: Some(Duration::default()),
			queue: VecDeque::from([Skill {
				data: Queued {
					slot_key: SlotKey::Hand(Side::Main),
					..default()
				},
				cast: Cast {
					aim: Duration::from_millis(42),
					pre: Duration::from_millis(1),
					active: Duration::from_millis(2),
					after: Duration::from_millis(3),
				},
				..default()
			}]),
			state: DequeueAble,
		};

		let manager = queue.get_active().unwrap();

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
					data: Queued {
						slot_key: SlotKey::Hand(Side::Main),
						..default()
					},
					cast: Cast {
						aim: Duration::from_millis(42),
						pre: Duration::from_millis(1),
						active: Duration::from_millis(2),
						after: Duration::from_millis(3),
					},
					..default()
				},
				Skill {
					data: Queued {
						slot_key: SlotKey::Hand(Side::Main),
						..default()
					},
					..default()
				},
			]),
			state: DequeueAble,
		};

		let manager = queue.get_active().unwrap();

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
				data: Queued {
					slot_key: SlotKey::Hand(Side::Main),
					..default()
				},
				..default()
			}]),
			state: DequeueAble,
		};

		let mut manager = queue.get_active().unwrap();

		assert_eq!(&mut Duration::from_secs(11), manager.elapsed_mut())
	}

	#[test]
	fn clear_duration_when_calling_clear() {
		let mut queue = QueueCollection {
			duration: Some(Duration::from_secs(11)),
			queue: VecDeque::from([Skill {
				data: Queued {
					slot_key: SlotKey::Hand(Side::Main),
					..default()
				},
				..default()
			}]),
			state: DequeueAble,
		};

		queue.clear_active();

		assert_eq!(None, queue.duration);
	}

	#[test]
	fn no_dequeue_when_duration_set() {
		let mut queue = QueueCollection {
			duration: Some(Duration::from_secs(11)),
			queue: VecDeque::from([Skill {
				data: Queued {
					slot_key: SlotKey::Hand(Side::Main),
					..default()
				},
				..default()
			}]),
			state: DequeueAble,
		};

		queue.try_dequeue();

		assert_eq!(
			VecDeque::from([Skill {
				data: Queued {
					slot_key: SlotKey::Hand(Side::Main),
					..default()
				},
				..default()
			}]),
			queue.queue
		);
	}

	#[test]
	fn set_default_duration_when_getting_manager() {
		let mut queue = QueueCollection {
			duration: None,
			queue: VecDeque::from([Skill {
				data: Queued {
					slot_key: SlotKey::Hand(Side::Main),
					..default()
				},
				..default()
			}]),
			state: DequeueAble,
		};

		assert_eq!(
			(Some(Duration::default()), Some(Duration::default())),
			(
				queue.get_active().map(|mut m| *m.elapsed_mut()),
				queue.duration
			)
		);
	}

	#[test]
	fn do_not_set_default_duration_when_getting_manager_when_queue_is_empty() {
		let mut queue = QueueCollection {
			duration: None,
			queue: default(),
			state: DequeueAble,
		};

		queue.get_active();

		assert_eq!(None, queue.duration);
	}

	#[test]
	fn test_start_behavior_fn() {
		fn run(_: &mut EntityCommands, _: &Transform, _: &Spawner, _: &Target) {}

		let active = ActiveSkill {
			skill: &Skill {
				data: Queued {
					slot_key: SlotKey::Hand(Side::Main),
					..default()
				},
				execution: SkillExecution {
					run_fn: Some(run),
					..default()
				},
				..default()
			},
			duration: &mut Duration::default(),
		};

		assert_eq!(Some(run as usize), active.get_start().map(|f| f as usize));
	}

	#[test]
	fn test_stop_behavior_fn() {
		fn stop(_: &mut EntityCommands) {}

		let active = ActiveSkill {
			skill: &Skill {
				data: Queued {
					slot_key: SlotKey::Hand(Side::Main),
					..default()
				},
				execution: SkillExecution {
					stop_fn: Some(stop),
					..default()
				},
				..default()
			},
			duration: &mut Duration::default(),
		};

		assert_eq!(Some(stop as usize), active.get_stop().map(|f| f as usize));
	}

	#[test]
	fn get_shoot_animations() {
		let actives_main = [
			ActiveSkill {
				duration: &mut Duration::default(),
				skill: &Skill {
					data: Queued {
						slot_key: SlotKey::Hand(Side::Main),
						..default()
					},
					animate: Some(PlayerSkills::Shoot(Handed::Single(SideUnset))),
					..default()
				},
			},
			ActiveSkill {
				duration: &mut Duration::default(),
				skill: &Skill {
					data: Queued {
						slot_key: SlotKey::Hand(Side::Main),
						..default()
					},
					animate: Some(PlayerSkills::Shoot(Handed::Dual(SideUnset))),
					..default()
				},
			},
		];
		let actives_off = [
			ActiveSkill {
				duration: &mut Duration::default(),
				skill: &Skill {
					data: Queued {
						slot_key: SlotKey::Hand(Side::Off),
						..default()
					},
					animate: Some(PlayerSkills::Shoot(Handed::Single(SideUnset))),
					..default()
				},
			},
			ActiveSkill {
				duration: &mut Duration::default(),
				skill: &Skill {
					data: Queued {
						slot_key: SlotKey::Hand(Side::Off),
						..default()
					},
					animate: Some(PlayerSkills::Shoot(Handed::Dual(SideUnset))),
					..default()
				},
			},
		];

		assert_eq!(
			(
				[
					Animate::Repeat(PlayerSkills::Shoot(Handed::Single(Side::Main))),
					Animate::Repeat(PlayerSkills::Shoot(Handed::Dual(Side::Main)))
				],
				[
					Animate::Repeat(PlayerSkills::Shoot(Handed::Single(Side::Off))),
					Animate::Repeat(PlayerSkills::Shoot(Handed::Dual(Side::Off)))
				],
			),
			(
				actives_main.map(|track| track.animate()),
				actives_off.map(|track| track.animate())
			)
		)
	}

	#[test]
	fn get_sword_strike_animations() {
		let animate = PlayerSkills::SwordStrike(SideUnset);
		let active_main = ActiveSkill {
			duration: &mut Duration::default(),
			skill: &Skill {
				data: Queued {
					slot_key: SlotKey::Hand(Side::Main),
					..default()
				},
				animate: Some(animate),
				..default()
			},
		};
		let active_off = ActiveSkill {
			duration: &mut Duration::default(),
			skill: &Skill {
				data: Queued {
					slot_key: SlotKey::Hand(Side::Off),
					..default()
				},
				animate: Some(animate),
				..default()
			},
		};

		assert_eq!(
			[
				Animate::Replay(PlayerSkills::SwordStrike(Side::Main)),
				Animate::Replay(PlayerSkills::SwordStrike(Side::Off))
			],
			[active_main.animate(), active_off.animate()]
		)
	}

	#[test]
	fn get_main_slot() {
		let active = ActiveSkill {
			duration: &mut Duration::default(),
			skill: &Skill {
				data: Queued {
					slot_key: SlotKey::Hand(Side::Off),
					..default()
				},
				..default()
			},
		};

		assert_eq!(vec![SlotKey::Hand(Side::Off)], active.slots());
	}

	#[test]
	fn get_off_slot() {
		let active = ActiveSkill {
			duration: &mut Duration::default(),
			skill: &Skill {
				data: Queued {
					slot_key: SlotKey::Hand(Side::Main),
					..default()
				},
				..default()
			},
		};

		assert_eq!(vec![SlotKey::Hand(Side::Main)], active.slots());
	}

	#[test]
	fn get_dual_main_slots() {
		let active = ActiveSkill {
			duration: &mut Duration::default(),
			skill: &Skill {
				data: Queued {
					slot_key: SlotKey::Hand(Side::Main),
					..default()
				},
				dual_wield: true,
				..default()
			},
		};

		assert_eq!(
			vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)],
			active.slots()
		);
	}

	#[test]
	fn get_dual_off_slots() {
		let active = ActiveSkill {
			duration: &mut Duration::default(),
			skill: &Skill {
				data: Queued {
					slot_key: SlotKey::Hand(Side::Off),
					..default()
				},
				dual_wield: true,
				..default()
			},
		};

		assert_eq!(
			vec![SlotKey::Hand(Side::Off), SlotKey::Hand(Side::Main)],
			active.slots()
		);
	}

	#[test]
	fn get_skill_spawn_slot() {
		let active = ActiveSkill {
			duration: &mut Duration::default(),
			skill: &Skill {
				data: Queued {
					slot_key: SlotKey::SkillSpawn,
					..default()
				},
				dual_wield: true,
				..default()
			},
		};

		assert_eq!(vec![SlotKey::SkillSpawn], active.slots());
	}
}
