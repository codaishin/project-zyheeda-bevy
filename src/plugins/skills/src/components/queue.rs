pub(crate) mod dto;

use crate::{
	QueueDto,
	components::held_slots::{Current, HeldSlots, Old},
	skills::{QueuedSkill, RunSkillBehavior, Skill, SkillMode, SkillState},
	traits::{Enqueue, Flush, GetActiveSkill, GetSkillBehavior, IterAddedMut, IterHoldingMut},
};
use bevy::prelude::*;
use common::{
	tools::action_key::slot::SlotKey,
	traits::{iterate::Iterate, state_duration::StateDuration},
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};
use std::{
	collections::{VecDeque, vec_deque::Iter},
	time::Duration,
};

#[derive(PartialEq, Debug, Default, Clone, Serialize, Deserialize)]
enum State {
	#[default]
	Flushed,
	Changed {
		len_before_change: usize,
	},
}

#[derive(Component, SavableComponent, PartialEq, Debug, Clone)]
#[savable_component(dto = QueueDto)]
#[require(HeldSlots<Current>, HeldSlots<Old>)]
pub struct Queue {
	queue: VecDeque<QueuedSkill>,
	active: Option<SkillElapsed<Duration>>,
	state: State,
}

impl Queue {
	fn unchanged_length(&self) -> usize {
		let Queue { queue, state, .. } = self;
		match state {
			State::Flushed => queue.len(),
			State::Changed { len_before_change } => *len_before_change,
		}
	}
}

impl Default for Queue {
	fn default() -> Self {
		Self {
			queue: VecDeque::from([]),
			active: None,
			state: State::Flushed,
		}
	}
}

#[cfg(test)]
impl<T> From<T> for Queue
where
	T: IntoIterator<Item = QueuedSkill>,
{
	fn from(skills: T) -> Self {
		Self {
			queue: VecDeque::from_iter(skills),
			active: None,
			state: State::Flushed,
		}
	}
}

impl<'a> Iterate<'a> for Queue {
	type TItem = &'a QueuedSkill;
	type TIter = Iter<'a, QueuedSkill>;

	fn iterate(&'a self) -> Self::TIter {
		self.queue.iter()
	}
}

impl Enqueue<(Skill, SlotKey)> for Queue {
	fn enqueue(&mut self, item: (Skill, SlotKey)) {
		if self.state == State::Flushed {
			let len_before_change = self.queue.len();
			self.state = State::Changed { len_before_change }
		}

		let (skill, slot_key) = item;

		self.queue.push_back(QueuedSkill::new(skill, slot_key));
	}
}

impl Flush for Queue {
	fn flush(&mut self) {
		self.state = State::Flushed;

		if self.active.is_some() {
			return;
		}

		self.queue.pop_front();
	}
}

impl IterHoldingMut for Queue {
	type TItem = QueuedSkill;

	fn iter_holding_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut QueuedSkill>
	where
		QueuedSkill: 'a,
	{
		self.queue
			.iter_mut()
			.filter(|skill| skill.skill_mode == SkillMode::Hold)
	}
}

impl IterAddedMut for Queue {
	type TItem = QueuedSkill;

	fn added_none(&self) -> bool {
		self.unchanged_length() == self.queue.len()
	}

	fn iter_added_mut<'a>(&'a mut self) -> impl DoubleEndedIterator<Item = &'a mut QueuedSkill>
	where
		QueuedSkill: 'a,
	{
		let unchanged_length = self.unchanged_length();

		self.queue.iter_mut().skip(unchanged_length)
	}
}

impl GetActiveSkill<SkillState> for Queue {
	type TActive<'a>
		= ActiveSkill<'a>
	where
		Self: 'a;

	fn get_active(&mut self) -> Option<Self::TActive<'_>> {
		let skill = self.queue.front_mut()?;
		let elapsed = self.active.get_or_insert_default();

		if skill.skill_mode == SkillMode::Release && elapsed.released.is_none() {
			elapsed.released = Some(elapsed.active);
		}

		Some(ActiveSkill { skill, elapsed })
	}

	fn clear_active(&mut self) {
		self.active = None;
	}
}

#[derive(PartialEq, Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct SkillElapsed<TDuration> {
	active: TDuration,
	#[serde(skip_serializing_if = "Option::is_none")]
	released: Option<TDuration>,
}

impl Default for SkillElapsed<Duration> {
	fn default() -> Self {
		Self {
			active: Duration::ZERO,
			released: None,
		}
	}
}

pub(crate) struct ActiveSkill<'a> {
	skill: &'a mut QueuedSkill,
	elapsed: &'a mut SkillElapsed<Duration>,
}

impl StateDuration<SkillState> for ActiveSkill<'_> {
	fn get_state_duration(&self, key: SkillState) -> Duration {
		match (key, &self.skill.skill_mode) {
			(SkillState::Aim, SkillMode::Hold) => Duration::MAX,
			(SkillState::Aim, SkillMode::Release) => self.elapsed.released.unwrap_or(Duration::MAX),
			(SkillState::Active, _) => self.skill.skill.cast_time,
		}
	}

	fn elapsed(&self) -> Duration {
		self.elapsed.active
	}

	fn set_elapsed(&mut self, elapsed: Duration) {
		self.elapsed.active = elapsed;
	}
}

impl GetSkillBehavior for ActiveSkill<'_> {
	fn behavior(&self) -> (SlotKey, RunSkillBehavior) {
		(self.skill.key, self.skill.skill.behavior.clone())
	}
}

#[cfg(test)]
mod test_queue_collection {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use bevy::utils::default;
	use common::traits::handles_localization::Token;

	#[test]
	fn enqueue_one_skill() {
		let mut queue = Queue::from([]);
		queue.enqueue((
			Skill {
				token: Token::from("my skill"),
				..default()
			},
			SlotKey(11),
		));

		assert_eq!(
			VecDeque::from([QueuedSkill {
				skill: Skill {
					token: Token::from("my skill"),
					..default()
				},
				key: SlotKey(11),
				..default()
			}]),
			queue.queue
		);
	}

	#[test]
	fn enqueue_two_skills() {
		let mut queue = Queue::from([]);
		queue.enqueue((
			Skill {
				token: Token::from("skill a"),
				..default()
			},
			SlotKey(42),
		));
		queue.enqueue((
			Skill {
				token: Token::from("skill b"),
				..default()
			},
			SlotKey(11),
		));

		assert_eq!(
			VecDeque::from([
				QueuedSkill {
					key: SlotKey(42),
					skill: Skill {
						token: Token::from("skill a"),
						..default()
					},
					..default()
				},
				QueuedSkill {
					key: SlotKey(11),
					skill: Skill {
						token: Token::from("skill b"),
						..default()
					},
					..default()
				},
			]),
			queue.queue
		);
	}

	#[test]
	fn flush_with_one_skill() {
		let mut queue = Queue::from([QueuedSkill {
			key: SlotKey(11),
			skill: Skill {
				token: Token::from("my skill"),
				..default()
			},
			..default()
		}]);

		queue.flush();

		assert_eq!(Queue::from([]), queue);
	}

	#[test]
	fn flush_with_two_skill() {
		let mut queue = Queue::from([
			QueuedSkill {
				key: SlotKey(42),
				skill: Skill {
					token: Token::from("skill a"),
					..default()
				},
				..default()
			},
			QueuedSkill {
				key: SlotKey(11),
				skill: Skill {
					token: Token::from("skill b"),
					..default()
				},
				..default()
			},
		]);

		queue.flush();

		let queue_after_1_flush = Queue {
			queue: queue.queue.clone(),
			active: queue.active,
			state: queue.state.clone(),
		};

		queue.flush();

		let queue_after_2_flushes = Queue {
			queue: queue.queue.clone(),
			active: queue.active,
			state: queue.state.clone(),
		};

		assert_eq!(
			(
				Queue::from([QueuedSkill {
					key: SlotKey(11),
					skill: Skill {
						token: Token::from("skill b"),
						..default()
					},
					..default()
				}]),
				Queue::from([])
			),
			(queue_after_1_flush, queue_after_2_flushes)
		);
	}

	#[test]
	fn iter() {
		let queue = Queue::from([
			QueuedSkill {
				key: SlotKey(42),
				skill: Skill {
					token: Token::from("skill a"),
					..default()
				},
				..default()
			},
			QueuedSkill {
				key: SlotKey(11),
				skill: Skill {
					token: Token::from("skill b"),
					..default()
				},
				..default()
			},
		]);

		assert_eq!(
			vec![
				&QueuedSkill {
					key: SlotKey(42),
					skill: Skill {
						token: Token::from("skill a"),
						..default()
					},
					..default()
				},
				&QueuedSkill {
					key: SlotKey(11),
					skill: Skill {
						token: Token::from("skill b"),
						..default()
					},
					..default()
				},
			],
			queue.iterate().collect::<Vec<_>>()
		)
	}

	#[test]
	fn iter_recent_mut() {
		let mut queue = Queue::from([]);
		queue.enqueue((
			Skill {
				token: Token::from("a"),

				..default()
			},
			SlotKey(11),
		));
		queue.enqueue((
			Skill {
				token: Token::from("b"),
				..default()
			},
			SlotKey(42),
		));

		assert_eq!(
			(
				false,
				vec![
					&mut QueuedSkill {
						key: SlotKey(11),
						skill: Skill {
							token: Token::from("a"),
							..default()
						},
						..default()
					},
					&mut QueuedSkill {
						key: SlotKey(42),
						skill: Skill {
							token: Token::from("b"),
							..default()
						},
						..default()
					},
				]
			),
			(
				queue.added_none(),
				queue.iter_added_mut().collect::<Vec<_>>()
			)
		)
	}

	#[test]
	fn iter_recent_mut_only_new() {
		let mut queue = Queue::from([QueuedSkill {
			key: SlotKey(11),
			skill: Skill {
				token: Token::from("a"),
				..default()
			},
			..default()
		}]);
		queue.enqueue((
			Skill {
				token: Token::from("b"),

				..default()
			},
			SlotKey(11),
		));
		queue.enqueue((
			Skill {
				token: Token::from("c"),

				..default()
			},
			SlotKey(42),
		));

		assert_eq!(
			(
				false,
				vec![
					&mut QueuedSkill {
						key: SlotKey(11),
						skill: Skill {
							token: Token::from("b"),
							..default()
						},
						..default()
					},
					&mut QueuedSkill {
						key: SlotKey(42),
						skill: Skill {
							token: Token::from("c"),
							..default()
						},
						..default()
					},
				],
			),
			(
				queue.added_none(),
				queue.iter_added_mut().collect::<Vec<_>>()
			)
		)
	}

	#[test]
	fn iter_recent_mut_empty_after_flush() {
		let mut queue = Queue::from([QueuedSkill {
			key: SlotKey(11),
			skill: Skill {
				token: Token::from("a"),
				..default()
			},
			..default()
		}]);
		queue.enqueue((
			Skill {
				token: Token::from("b"),

				..default()
			},
			SlotKey(11),
		));
		queue.enqueue((
			Skill {
				token: Token::from("c"),

				..default()
			},
			SlotKey(42),
		));

		queue.flush();

		assert_eq!(
			(true, vec![] as Vec<&mut QueuedSkill>),
			(
				queue.added_none(),
				queue.iter_added_mut().collect::<Vec<_>>()
			)
		)
	}

	#[test]
	fn iter_recent_mut_empty_after_flush_with_active_duration() {
		let mut queue = Queue::from([QueuedSkill {
			key: SlotKey(11),
			skill: Skill {
				token: Token::from("a"),
				..default()
			},
			..default()
		}]);
		queue.enqueue((
			Skill {
				token: Token::from("b"),

				..default()
			},
			SlotKey(11),
		));
		queue.enqueue((
			Skill {
				token: Token::from("c"),

				..default()
			},
			SlotKey(42),
		));

		queue.active = Some(SkillElapsed::default());
		queue.flush();

		assert_eq!(
			(true, vec![] as Vec<&mut QueuedSkill>,),
			(
				queue.added_none(),
				queue.iter_added_mut().collect::<Vec<_>>()
			)
		)
	}

	#[test]
	fn get_holding_mut() {
		let skills = [
			QueuedSkill {
				key: SlotKey(11),
				skill: Skill {
					token: Token::from("holding"),
					..default()
				},
				skill_mode: SkillMode::Hold,
			},
			QueuedSkill {
				key: SlotKey(11),
				skill: Skill {
					token: Token::from("active"),
					..default()
				},
				skill_mode: SkillMode::Release,
			},
		];
		let mut queue = Queue::from(skills.clone());

		assert_eq!(
			vec![&skills[0]],
			queue.iter_holding_mut().collect::<Vec<_>>()
		);
	}
}

#[cfg(test)]
mod test_queue_active_skill {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::{
		behaviors::{
			SkillBehaviorConfig,
			spawn_skill::{OnSkillStop, SpawnSkill},
		},
		skills::RunSkillBehavior,
		traits::skill_builder::SkillShape,
	};

	#[test]
	fn get_phase_times_for_holding() {
		let mut queue = Queue {
			active: Some(SkillElapsed {
				active: Duration::from_millis(42),
				..default()
			}),
			queue: VecDeque::from([
				QueuedSkill {
					skill: Skill {
						cast_time: Duration::from_millis(1),
						..default()
					},
					key: SlotKey(11),
					skill_mode: SkillMode::Hold,
				},
				QueuedSkill::default(),
			]),
			..default()
		};

		let manager = queue.get_active().unwrap();

		assert_eq!(
			[Duration::MAX, Duration::from_millis(1)],
			[
				manager.get_state_duration(SkillState::Aim),
				manager.get_state_duration(SkillState::Active),
			]
		)
	}

	#[test]
	fn get_phase_times_for_released() {
		let mut queue = Queue {
			active: Some(SkillElapsed {
				active: Duration::from_millis(42),
				released: Some(Duration::from_millis(100)),
			}),
			queue: VecDeque::from([
				QueuedSkill {
					skill: Skill {
						cast_time: Duration::from_millis(1),
						..default()
					},
					key: SlotKey(11),
					skill_mode: SkillMode::Release,
				},
				QueuedSkill::default(),
			]),
			..default()
		};

		let active = queue.get_active().unwrap();

		assert_eq!(
			[Duration::from_millis(100), Duration::from_millis(1)],
			[
				active.get_state_duration(SkillState::Aim),
				active.get_state_duration(SkillState::Active),
			]
		)
	}

	#[test]
	fn get_phase_times_for_newly_released() {
		let mut queue = Queue {
			active: Some(SkillElapsed {
				active: Duration::from_millis(42),
				..default()
			}),
			queue: VecDeque::from([
				QueuedSkill {
					skill: Skill {
						cast_time: Duration::from_millis(1),
						..default()
					},
					key: SlotKey(11),
					skill_mode: SkillMode::Release,
				},
				QueuedSkill::default(),
			]),
			..default()
		};

		let active = queue.get_active().unwrap();

		assert_eq!(
			[Duration::from_millis(42), Duration::from_millis(1)],
			[
				active.get_state_duration(SkillState::Aim),
				active.get_state_duration(SkillState::Active),
			]
		)
	}

	#[test]
	fn set_elapsed_for_newly_released() {
		let mut queue = Queue {
			active: Some(SkillElapsed {
				active: Duration::from_millis(42),
				..default()
			}),
			queue: VecDeque::from([
				QueuedSkill {
					skill: Skill {
						cast_time: Duration::from_millis(1),
						..default()
					},
					key: SlotKey(11),
					skill_mode: SkillMode::Release,
				},
				QueuedSkill::default(),
			]),
			..default()
		};

		_ = queue.get_active();

		assert_eq!(
			Some(SkillElapsed {
				active: Duration::from_millis(42),
				released: Some(Duration::from_millis(42)),
			}),
			queue.active
		)
	}

	#[test]
	fn get_duration() {
		let mut queue = Queue {
			active: Some(SkillElapsed {
				active: Duration::from_millis(11),
				..default()
			}),
			queue: VecDeque::from([QueuedSkill {
				key: SlotKey(11),
				..default()
			}]),
			..default()
		};

		let active = queue.get_active().unwrap();

		assert_eq!(Duration::from_millis(11), active.elapsed())
	}

	#[test]
	fn clear_duration_when_calling_clear() {
		let mut queue = Queue {
			active: Some(SkillElapsed::default()),
			queue: VecDeque::from([QueuedSkill {
				key: SlotKey(11),
				..default()
			}]),
			..default()
		};

		queue.clear_active();

		assert_eq!(None, queue.active);
	}

	#[test]
	fn do_not_pop_front_on_flush_when_duration_set() {
		let mut queue = Queue {
			active: Some(SkillElapsed::default()),
			queue: VecDeque::from([QueuedSkill {
				key: SlotKey(11),
				..default()
			}]),
			..default()
		};

		queue.flush();

		assert_eq!(
			VecDeque::from([QueuedSkill {
				key: SlotKey(11),
				..default()
			}]),
			queue.queue
		);
	}

	#[test]
	fn set_default_duration_when_getting_manager() {
		let mut queue = Queue {
			active: None,
			queue: VecDeque::from([QueuedSkill {
				key: SlotKey(11),
				..default()
			}]),
			..default()
		};

		assert_eq!(
			(Some(Duration::default()), Some(SkillElapsed::default())),
			(queue.get_active().map(|m| m.elapsed()), queue.active)
		);
	}

	#[test]
	fn do_not_set_default_duration_when_getting_manager_when_queue_is_empty() {
		let mut queue = Queue {
			queue: VecDeque::from([]),
			..default()
		};

		queue.get_active();

		assert_eq!(None, queue.active);
	}

	#[test]
	fn test_start_behavior_fn_on_active() {
		let behaviors =
			SkillBehaviorConfig::from_shape(SpawnSkill::Fn(|commands, _, _, _| SkillShape {
				contact: commands.spawn(()).id(),
				projection: commands.spawn(()).id(),
				on_skill_stop: OnSkillStop::Ignore,
			}));

		let active = ActiveSkill {
			skill: &mut QueuedSkill {
				skill: Skill {
					behavior: RunSkillBehavior::OnActive(behaviors.clone()),
					..default()
				},
				key: SlotKey(42),
				..default()
			},
			elapsed: &mut SkillElapsed::default(),
		};

		assert_eq!(
			(SlotKey(42), RunSkillBehavior::OnActive(behaviors)),
			active.behavior()
		);
	}

	#[test]
	fn test_start_behavior_fn_on_aim() {
		let behaviors =
			SkillBehaviorConfig::from_shape(SpawnSkill::Fn(|commands, _, _, _| SkillShape {
				contact: commands.spawn(()).id(),
				projection: commands.spawn(()).id(),
				on_skill_stop: OnSkillStop::Ignore,
			}));

		let active = ActiveSkill {
			skill: &mut QueuedSkill {
				skill: Skill {
					behavior: RunSkillBehavior::OnAim(behaviors.clone()),
					..default()
				},
				key: SlotKey(86),
				..default()
			},
			elapsed: &mut SkillElapsed::default(),
		};

		assert_eq!(
			(SlotKey(86), RunSkillBehavior::OnAim(behaviors)),
			active.behavior()
		);
	}
}
