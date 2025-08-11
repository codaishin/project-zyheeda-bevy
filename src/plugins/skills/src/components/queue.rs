pub(crate) mod dto;

use crate::{
	QueueDto,
	skills::{Activation, AnimationStrategy, QueuedSkill, RunSkillBehavior, Skill, SkillState},
	traits::{
		Enqueue,
		Flush,
		GetActiveSkill,
		GetAnimationStrategy,
		GetSkillBehavior,
		IterAddedMut,
		IterWaitingMut,
	},
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
pub struct Queue {
	queue: VecDeque<QueuedSkill>,
	duration: Option<Duration>,
	state: State,
}

impl Default for Queue {
	fn default() -> Self {
		Self {
			queue: default(),
			duration: default(),
			state: default(),
		}
	}
}

#[cfg(test)]
impl Queue {
	pub fn new<const N: usize>(items: [QueuedSkill; N]) -> Self {
		Self {
			queue: VecDeque::from(items),
			duration: None,
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

		if self.duration.is_some() {
			return;
		}

		self.queue.pop_front();
	}
}

impl IterWaitingMut<QueuedSkill> for Queue {
	fn iter_waiting_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut QueuedSkill>
	where
		QueuedSkill: 'a,
	{
		self.queue
			.iter_mut()
			.filter(|skill| skill.mode == Activation::Waiting)
	}
}

impl IterAddedMut for Queue {
	type TItem = QueuedSkill;

	fn added_none(&self) -> bool {
		unchanged_length(self) == self.queue.len()
	}

	fn iter_added_mut<'a>(&'a mut self) -> impl DoubleEndedIterator<Item = &'a mut QueuedSkill>
	where
		QueuedSkill: 'a,
	{
		let unchanged_length = unchanged_length(self);

		self.queue.iter_mut().skip(unchanged_length)
	}
}

fn unchanged_length(Queue { queue, state, .. }: &Queue) -> usize {
	match state {
		State::Flushed => queue.len(),
		State::Changed { len_before_change } => *len_before_change,
	}
}

#[cfg(test)]
mod test_queue_collection {
	use super::*;
	use bevy::utils::default;
	use common::traits::handles_localization::Token;

	#[test]
	fn enqueue_one_skill() {
		let mut queue = Queue::new([]);
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
		let mut queue = Queue::new([]);
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
		let mut queue = Queue::new([QueuedSkill {
			key: SlotKey(11),
			skill: Skill {
				token: Token::from("my skill"),
				..default()
			},
			..default()
		}]);

		queue.flush();

		assert_eq!(Queue::new([]), queue);
	}

	#[test]
	fn flush_with_two_skill() {
		let mut queue = Queue::new([
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
			duration: queue.duration,
			state: queue.state.clone(),
		};

		queue.flush();

		let queue_after_2_flushes = Queue {
			queue: queue.queue.clone(),
			duration: queue.duration,
			state: queue.state.clone(),
		};

		assert_eq!(
			(
				Queue::new([QueuedSkill {
					key: SlotKey(11),
					skill: Skill {
						token: Token::from("skill b"),
						..default()
					},
					..default()
				}]),
				Queue::new([])
			),
			(queue_after_1_flush, queue_after_2_flushes)
		);
	}

	#[test]
	fn iter() {
		let queue = Queue::new([
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
		let mut queue = Queue::new([]);
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
		let mut queue = Queue::new([QueuedSkill {
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
		let mut queue = Queue::new([QueuedSkill {
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
		let mut queue = Queue::new([QueuedSkill {
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

		queue.duration = Some(Duration::from_millis(42));
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
	fn iter_waiting() {
		let skills = [
			QueuedSkill {
				key: SlotKey(11),
				skill: Skill {
					token: Token::from("active"),
					..default()
				},
				mode: Activation::ActiveAfter(Duration::from_millis(100)),
			},
			QueuedSkill {
				key: SlotKey(11),
				skill: Skill {
					token: Token::from("waiting"),
					..default()
				},
				mode: Activation::Waiting,
			},
			QueuedSkill {
				key: SlotKey(11),
				skill: Skill {
					token: Token::from("primed"),
					..default()
				},
				mode: Activation::Primed,
			},
		];
		let mut queue = Queue::new(skills.clone());

		assert_eq!(
			vec![&skills[1]],
			queue.iter_waiting_mut().collect::<Vec<_>>()
		);
	}
}

struct ActiveSkill<'a> {
	skill: &'a Skill,
	slot_key: &'a SlotKey,
	mode: &'a mut Activation,
	duration: &'a mut Duration,
}

impl GetActiveSkill<SkillState> for Queue {
	#[allow(refining_impl_trait)]
	fn get_active(
		&mut self,
	) -> Option<impl GetSkillBehavior + GetAnimationStrategy + StateDuration<SkillState>> {
		let skill = self.queue.front_mut()?;

		if self.duration.is_none() {
			self.duration = Some(Duration::default());
		}

		Some(ActiveSkill {
			skill: &skill.skill,
			slot_key: &skill.key,
			mode: &mut skill.mode,
			duration: self.duration.as_mut()?,
		})
	}

	fn clear_active(&mut self) {
		self.duration = None;
	}
}

impl StateDuration<SkillState> for ActiveSkill<'_> {
	fn get_state_duration(&self, key: SkillState) -> Duration {
		match (key, &self.mode) {
			(SkillState::Aim, Activation::Primed | Activation::Waiting) => Duration::MAX,
			(SkillState::Aim, Activation::ActiveAfter(duration)) => *duration,
			(SkillState::Active, _) => self.skill.cast_time,
		}
	}

	fn elapsed_mut(&mut self) -> &mut Duration {
		if let Activation::Primed = self.mode {
			*self.mode = Activation::ActiveAfter(*self.duration)
		}
		self.duration
	}
}

impl GetSkillBehavior for ActiveSkill<'_> {
	fn behavior(&self) -> (SlotKey, RunSkillBehavior) {
		(*self.slot_key, self.skill.behavior.clone())
	}
}

impl GetAnimationStrategy for ActiveSkill<'_> {
	fn animation_strategy(&self) -> AnimationStrategy {
		self.skill.animation
	}
}

#[cfg(test)]
mod test_queue_active_skill {
	use super::*;
	use crate::{
		behaviors::{
			SkillBehaviorConfig,
			build_skill_shape::{BuildSkillShape, OnSkillStop},
		},
		skills::{AnimationStrategy, RunSkillBehavior},
		traits::skill_builder::SkillShape,
	};
	use test_case::test_case;

	#[test]
	fn get_phasing_times_waiting() {
		let mut queue = Queue {
			queue: VecDeque::from([
				QueuedSkill {
					skill: Skill {
						cast_time: Duration::from_millis(1),
						..default()
					},
					key: SlotKey(11),
					mode: Activation::Waiting,
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
	fn get_phasing_times_primed() {
		let mut queue = Queue {
			queue: VecDeque::from([
				QueuedSkill {
					skill: Skill {
						cast_time: Duration::from_millis(1),
						..default()
					},
					key: SlotKey(11),
					mode: Activation::Primed,
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
	fn get_phasing_times_active() {
		let mut queue = Queue {
			queue: VecDeque::from([
				QueuedSkill {
					skill: Skill {
						cast_time: Duration::from_millis(1),
						..default()
					},
					key: SlotKey(11),
					mode: Activation::ActiveAfter(Duration::from_millis(42)),
				},
				QueuedSkill::default(),
			]),
			..default()
		};

		let manager = queue.get_active().unwrap();

		assert_eq!(
			[Duration::from_millis(42), Duration::from_millis(1)],
			[
				manager.get_state_duration(SkillState::Aim),
				manager.get_state_duration(SkillState::Active),
			]
		)
	}

	#[test]
	fn get_duration() {
		let mut queue = Queue {
			duration: Some(Duration::from_secs(11)),
			queue: VecDeque::from([QueuedSkill {
				key: SlotKey(11),
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
			queue: VecDeque::from([QueuedSkill {
				key: SlotKey(11),
				mode: Activation::Primed,
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
			queue.queue.front().unwrap().mode
		)
	}

	#[test]
	fn clear_duration_when_calling_clear() {
		let mut queue = Queue {
			duration: Some(Duration::from_secs(11)),
			queue: VecDeque::from([QueuedSkill {
				key: SlotKey(11),
				..default()
			}]),
			..default()
		};

		queue.clear_active();

		assert_eq!(None, queue.duration);
	}

	#[test]
	fn do_not_pop_front_on_flush_when_duration_set() {
		let mut queue = Queue {
			duration: Some(Duration::from_secs(11)),
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
			duration: None,
			queue: VecDeque::from([QueuedSkill {
				key: SlotKey(11),
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
	fn test_start_behavior_fn_on_active() {
		let behaviors =
			SkillBehaviorConfig::from_shape(BuildSkillShape::Fn(|commands, _, _, _| SkillShape {
				contact: commands.spawn(()).id(),
				projection: commands.spawn(()).id(),
				on_skill_stop: OnSkillStop::Ignore,
			}));

		let active = ActiveSkill {
			skill: &Skill {
				behavior: RunSkillBehavior::OnActive(behaviors.clone()),
				..default()
			},
			slot_key: &SlotKey(42),
			mode: &mut Activation::default(),
			duration: &mut Duration::default(),
		};

		assert_eq!(
			(SlotKey(42), RunSkillBehavior::OnActive(behaviors)),
			active.behavior()
		);
	}

	#[test]
	fn test_start_behavior_fn_on_aim() {
		let behaviors =
			SkillBehaviorConfig::from_shape(BuildSkillShape::Fn(|commands, _, _, _| SkillShape {
				contact: commands.spawn(()).id(),
				projection: commands.spawn(()).id(),
				on_skill_stop: OnSkillStop::Ignore,
			}));

		let active = ActiveSkill {
			skill: &mut Skill {
				behavior: RunSkillBehavior::OnAim(behaviors.clone()),
				..default()
			},
			slot_key: &SlotKey(86),
			mode: &mut Activation::default(),
			duration: &mut Duration::default(),
		};

		assert_eq!(
			(SlotKey(86), RunSkillBehavior::OnAim(behaviors)),
			active.behavior()
		);
	}

	#[test_case(AnimationStrategy::DoNotAnimate; "do not animate")]
	#[test_case(AnimationStrategy::Animate; "animate")]
	#[test_case(AnimationStrategy::None; "none")]
	fn get_animation(animation: AnimationStrategy) {
		let active = ActiveSkill {
			skill: &Skill {
				animation,
				..default()
			},
			slot_key: &default(),
			mode: &mut default(),
			duration: &mut default(),
		};

		assert_eq!(animation, active.animation_strategy());
	}

	#[test]
	fn get_ignore_animation() {
		let active = ActiveSkill {
			skill: &mut Skill {
				animation: AnimationStrategy::None,
				..default()
			},
			slot_key: &SlotKey(11),
			mode: &mut Activation::default(),
			duration: &mut Duration::default(),
		};

		assert_eq!(AnimationStrategy::None, active.animation_strategy())
	}

	#[test]
	fn get_none_animation() {
		let active = ActiveSkill {
			skill: &Skill {
				animation: AnimationStrategy::None,
				..default()
			},
			slot_key: &SlotKey(11),
			mode: &mut Activation::default(),
			duration: &mut Duration::default(),
		};

		assert_eq!(AnimationStrategy::None, active.animation_strategy())
	}
}
