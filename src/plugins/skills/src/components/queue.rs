use super::SlotKey;
use crate::{
	skill::{Activation, PlayerSkills, Queued, Skill, SkillState, StartBehaviorFn, StopBehaviorFn},
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
use bevy::ecs::component::Component;
use common::{
	components::{Animate, Side},
	traits::state_duration::StateDuration,
};
use std::{collections::VecDeque, time::Duration};

#[derive(PartialEq, Debug, Default, Clone)]
enum State {
	#[default]
	Flushed,
	Changed {
		len_before_change: usize,
	},
}

#[derive(Component, PartialEq, Debug, Default)]
pub struct Queue {
	queue: VecDeque<Skill<Queued>>,
	duration: Option<Duration>,
	state: State,
}

#[cfg(test)]
impl Queue {
	pub fn new<const N: usize>(items: [Skill<Queued>; N]) -> Self {
		Self {
			queue: VecDeque::from(items),
			duration: None,
			state: State::Flushed,
		}
	}
}

impl Iter<Skill<Queued>> for Queue {
	fn iter<'a>(&'a self) -> impl DoubleEndedIterator<Item = &'a Skill<Queued>>
	where
		Skill<Queued>: 'a,
	{
		self.queue.iter()
	}
}

impl Enqueue<Skill<Queued>> for Queue {
	fn enqueue(&mut self, item: Skill<Queued>) {
		if self.state == State::Flushed {
			let len_before_change = self.queue.len();
			self.state = State::Changed { len_before_change }
		}
		self.queue.push_back(item);
	}
}

impl TryDequeue<Skill<Queued>> for Queue {
	fn try_dequeue(&mut self) {
		self.state = State::Flushed;

		if self.duration.is_some() {
			return;
		}

		self.queue.pop_front();
	}
}

impl IterMut<Skill<Queued>> for Queue {
	fn iter_mut<'a>(&'a mut self) -> impl DoubleEndedIterator<Item = &'a mut Skill<Queued>>
	where
		Skill<Queued>: 'a,
	{
		self.queue.iter_mut()
	}
}

impl IterRecentMut<Skill<Queued>> for Queue {
	fn iter_recent_mut<'a>(&'a mut self) -> impl DoubleEndedIterator<Item = &'a mut Skill<Queued>>
	where
		Skill<Queued>: 'a,
	{
		let old_len = match self.state {
			State::Flushed => self.queue.len(),
			State::Changed { len_before_change } => len_before_change,
		};

		self.queue.iter_mut().skip(old_len)
	}
}

impl GetOldLastMut<Skill<Queued>> for Queue {
	fn get_old_last_mut<'a>(&'a mut self) -> Option<&'a mut Skill<Queued>>
	where
		Skill<Queued>: 'a,
	{
		let old_len = match self.state {
			State::Flushed => self.queue.len(),
			State::Changed { len_before_change } => len_before_change,
		};

		self.queue.iter_mut().take(old_len).last()
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
		let mut queue = Queue::new([]);
		queue.enqueue(Skill {
			name: "my skill",
			data: Queued {
				slot_key: SlotKey::Hand(Side::Main),
				..default()
			},
			..default()
		});

		assert_eq!(
			VecDeque::from([Skill {
				name: "my skill",
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
	fn enqueue_two_skills() {
		let mut queue = Queue::new([]);
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
			VecDeque::from([
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
			queue.queue
		);
	}

	#[test]
	fn dequeue_one_skill() {
		let mut queue = Queue::new([Skill {
			name: "my skill",
			data: Queued {
				slot_key: SlotKey::Hand(Side::Main),
				..default()
			},
			..default()
		}]);

		queue.try_dequeue();

		assert_eq!(Queue::new([]), queue);
	}

	#[test]
	fn dequeue_two_skill() {
		let mut queue = Queue::new([
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

		let queue_after_1_dequeue = Queue {
			queue: queue.queue.clone(),
			duration: queue.duration,
			state: queue.state.clone(),
		};

		queue.try_dequeue();

		let queue_after_2_dequeues = Queue {
			queue: queue.queue.clone(),
			duration: queue.duration,
			state: queue.state.clone(),
		};

		assert_eq!(
			(
				Queue::new([Skill {
					name: "skill b",
					data: Queued {
						slot_key: SlotKey::Hand(Side::Main),
						..default()
					},
					..default()
				},]),
				Queue::new([])
			),
			(queue_after_1_dequeue, queue_after_2_dequeues)
		);
	}

	#[test]
	fn iter() {
		let queue = Queue::new([
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
	fn iter_mut() {
		let mut queue = Queue::new([]);
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
	fn iter_recent_mut() {
		let mut queue = Queue::new([]);
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
		let mut queue = Queue::new([Skill {
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
	fn iter_recent_mut_empty_after_flush() {
		let mut queue = Queue::new([Skill {
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

		queue.try_dequeue();

		assert_eq!(
			vec![] as Vec<&mut Skill<Queued>>,
			queue.iter_recent_mut().collect::<Vec<_>>()
		)
	}

	#[test]
	fn iter_recent_mut_empty_after_flush_with_active_duration() {
		let mut queue = Queue::new([Skill {
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

		queue.duration = Some(Duration::from_millis(42));
		queue.try_dequeue();

		assert_eq!(
			vec![] as Vec<&mut Skill<Queued>>,
			queue.iter_recent_mut().collect::<Vec<_>>()
		)
	}

	#[test]
	fn get_old_last_mut() {
		let mut queue = Queue::new([Skill {
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
		let mut queue = Queue::new([Skill {
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
	skill: &'a mut Skill<Queued>,
}

impl GetActiveSkill<PlayerSkills<Side>, SkillState> for Queue {
	fn get_active(
		&mut self,
	) -> Option<
		impl Execution + GetAnimation<PlayerSkills<Side>> + GetSlots + StateDuration<SkillState>,
	> {
		let skill = self.queue.front_mut()?;

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
		match (key, &self.skill.data.mode) {
			(SkillState::Aim, Activation::ActiveAfter(duration)) => *duration,
			(SkillState::Aim, _) => Duration::MAX,
			(SkillState::PreCast, _) => self.skill.cast.pre,
			(SkillState::Active, _) => self.skill.cast.active,
			(SkillState::AfterCast, _) => self.skill.cast.after,
		}
	}

	fn elapsed_mut(&mut self) -> &mut Duration {
		if let Activation::Primed = self.skill.data.mode {
			self.skill.data.mode = Activation::ActiveAfter(*self.duration)
		}
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
	use bevy::{ecs::system::EntityCommands, prelude::default, transform::components::Transform};
	use common::components::Side;

	#[test]
	fn get_phasing_times_waiting() {
		let mut queue = Queue {
			queue: VecDeque::from([
				Skill {
					data: Queued {
						slot_key: SlotKey::Hand(Side::Main),
						mode: Activation::Waiting,
					},
					cast: Cast {
						pre: Duration::from_millis(1),
						active: Duration::from_millis(2),
						after: Duration::from_millis(3),
					},
					..default()
				},
				Skill::default(),
			]),
			..default()
		};

		let manager = queue.get_active().unwrap();

		assert_eq!(
			[
				(Duration::MAX, SkillState::Aim),
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
	fn get_phasing_times_primed() {
		let mut queue = Queue {
			queue: VecDeque::from([
				Skill {
					data: Queued {
						slot_key: SlotKey::Hand(Side::Main),
						mode: Activation::Primed,
					},
					cast: Cast {
						pre: Duration::from_millis(1),
						active: Duration::from_millis(2),
						after: Duration::from_millis(3),
					},
					..default()
				},
				Skill::default(),
			]),
			..default()
		};

		let manager = queue.get_active().unwrap();

		assert_eq!(
			[
				(Duration::MAX, SkillState::Aim),
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
	fn get_phasing_times_active() {
		let mut queue = Queue {
			queue: VecDeque::from([
				Skill {
					data: Queued {
						slot_key: SlotKey::Hand(Side::Main),
						mode: Activation::ActiveAfter(Duration::from_millis(42)),
					},
					cast: Cast {
						pre: Duration::from_millis(1),
						active: Duration::from_millis(2),
						after: Duration::from_millis(3),
					},
					..default()
				},
				Skill::default(),
			]),
			..default()
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
		let mut queue = Queue {
			duration: Some(Duration::from_secs(11)),
			queue: VecDeque::from([Skill {
				data: Queued {
					slot_key: SlotKey::Hand(Side::Main),
					..default()
				},
				..default()
			}]),
			..default()
		};

		let mut manager = queue.get_active().unwrap();

		assert_eq!(&mut Duration::from_secs(11), manager.elapsed_mut())
	}

	#[test]
	fn if_first_skill_primed_set_active_with_current_duration() {
		let mut queue = Queue {
			duration: Some(Duration::from_secs(11)),
			queue: VecDeque::from([Skill {
				data: Queued {
					slot_key: SlotKey::Hand(Side::Main),
					mode: Activation::Primed,
				},
				..default()
			}]),
			..default()
		};

		{
			let mut manager = queue.get_active().unwrap();
			_ = manager.elapsed_mut();
		}

		assert_eq!(
			Activation::ActiveAfter(Duration::from_secs(11)),
			queue.queue.front().unwrap().data.mode
		)
	}

	#[test]
	fn clear_duration_when_calling_clear() {
		let mut queue = Queue {
			duration: Some(Duration::from_secs(11)),
			queue: VecDeque::from([Skill {
				data: Queued {
					slot_key: SlotKey::Hand(Side::Main),
					..default()
				},
				..default()
			}]),
			..default()
		};

		queue.clear_active();

		assert_eq!(None, queue.duration);
	}

	#[test]
	fn no_dequeue_when_duration_set() {
		let mut queue = Queue {
			duration: Some(Duration::from_secs(11)),
			queue: VecDeque::from([Skill {
				data: Queued {
					slot_key: SlotKey::Hand(Side::Main),
					..default()
				},
				..default()
			}]),
			..default()
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
		let mut queue = Queue {
			duration: None,
			queue: VecDeque::from([Skill {
				data: Queued {
					slot_key: SlotKey::Hand(Side::Main),
					..default()
				},
				..default()
			}]),
			..default()
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
		let mut queue = Queue {
			queue: VecDeque::from([]),
			..default()
		};

		queue.get_active();

		assert_eq!(None, queue.duration);
	}

	#[test]
	fn test_start_behavior_fn() {
		fn run(_: &mut EntityCommands, _: &Transform, _: &Spawner, _: &Target) {}

		let active = ActiveSkill {
			skill: &mut Skill {
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
			skill: &mut Skill {
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
				skill: &mut Skill {
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
				skill: &mut Skill {
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
				skill: &mut Skill {
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
				skill: &mut Skill {
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
			skill: &mut Skill {
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
			skill: &mut Skill {
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
			skill: &mut Skill {
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
			skill: &mut Skill {
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
			skill: &mut Skill {
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
			skill: &mut Skill {
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
			skill: &mut Skill {
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
