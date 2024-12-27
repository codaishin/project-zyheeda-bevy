use crate::{
	skills::{Activation, Animate, QueuedSkill, RunSkillBehavior, Skill, SkillState},
	traits::{
		Enqueue,
		Flush,
		GetActiveSkill,
		GetAnimation,
		GetSkillBehavior,
		IterAddedMut,
		IterMut,
	},
};
use bevy::{ecs::component::Component, utils::default};
use common::{
	tools::slot_key::{Side, SlotKey},
	traits::{animation::Animation, iterate::Iterate, state_duration::StateDuration},
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
	queue: VecDeque<QueuedSkill>,
	duration: Option<Duration>,
	state: State,
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

impl Iterate<QueuedSkill> for Queue {
	fn iterate<'a>(&'a self) -> impl DoubleEndedIterator<Item = &'a QueuedSkill>
	where
		QueuedSkill: 'a,
	{
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

		self.queue.push_back(QueuedSkill {
			skill,
			slot_key,
			..default()
		});
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

impl IterMut<QueuedSkill> for Queue {
	fn iter_mut<'a>(&'a mut self) -> impl DoubleEndedIterator<Item = &'a mut QueuedSkill>
	where
		QueuedSkill: 'a,
	{
		self.queue.iter_mut()
	}
}

impl IterAddedMut<QueuedSkill> for Queue {
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
	use common::tools::slot_key::Side;

	#[test]
	fn enqueue_one_skill() {
		let mut queue = Queue::new([]);
		queue.enqueue((
			Skill {
				name: "my skill".to_owned(),
				..default()
			},
			SlotKey::BottomHand(Side::Right),
		));

		assert_eq!(
			VecDeque::from([QueuedSkill {
				skill: Skill {
					name: "my skill".to_owned(),
					..default()
				},
				slot_key: SlotKey::BottomHand(Side::Right),
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
				name: "skill a".to_owned(),
				..default()
			},
			SlotKey::BottomHand(Side::Left),
		));
		queue.enqueue((
			Skill {
				name: "skill b".to_owned(),
				..default()
			},
			SlotKey::BottomHand(Side::Right),
		));

		assert_eq!(
			VecDeque::from([
				QueuedSkill {
					slot_key: SlotKey::BottomHand(Side::Left),
					skill: Skill {
						name: "skill a".to_owned(),
						..default()
					},
					..default()
				},
				QueuedSkill {
					slot_key: SlotKey::BottomHand(Side::Right),
					skill: Skill {
						name: "skill b".to_owned(),
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
			slot_key: SlotKey::BottomHand(Side::Right),
			skill: Skill {
				name: "my skill".to_owned(),
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
				slot_key: SlotKey::BottomHand(Side::Left),
				skill: Skill {
					name: "skill a".to_owned(),
					..default()
				},
				..default()
			},
			QueuedSkill {
				slot_key: SlotKey::BottomHand(Side::Right),
				skill: Skill {
					name: "skill b".to_owned(),
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
					slot_key: SlotKey::BottomHand(Side::Right),
					skill: Skill {
						name: "skill b".to_owned(),
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
				slot_key: SlotKey::BottomHand(Side::Left),
				skill: Skill {
					name: "skill a".to_owned(),
					..default()
				},
				..default()
			},
			QueuedSkill {
				slot_key: SlotKey::BottomHand(Side::Right),
				skill: Skill {
					name: "skill b".to_owned(),
					..default()
				},
				..default()
			},
		]);

		assert_eq!(
			vec![
				&QueuedSkill {
					slot_key: SlotKey::BottomHand(Side::Left),
					skill: Skill {
						name: "skill a".to_owned(),
						..default()
					},
					..default()
				},
				&QueuedSkill {
					slot_key: SlotKey::BottomHand(Side::Right),
					skill: Skill {
						name: "skill b".to_owned(),
						..default()
					},
					..default()
				},
			],
			queue.iterate().collect::<Vec<_>>()
		)
	}

	#[test]
	fn iter_mut_with_keys() {
		let mut queue = Queue::new([]);
		queue.enqueue((
			Skill {
				name: "skill a".to_owned(),
				..default()
			},
			SlotKey::BottomHand(Side::Left),
		));
		queue.enqueue((
			Skill {
				name: "skill b".to_owned(),
				..default()
			},
			SlotKey::BottomHand(Side::Right),
		));

		assert_eq!(
			vec![
				&mut QueuedSkill {
					slot_key: SlotKey::BottomHand(Side::Left),
					skill: Skill {
						name: "skill a".to_owned(),
						..default()
					},
					..default()
				},
				&mut QueuedSkill {
					slot_key: SlotKey::BottomHand(Side::Right),
					skill: Skill {
						name: "skill b".to_owned(),
						..default()
					},
					..default()
				},
			],
			queue.iter_mut().collect::<Vec<_>>()
		)
	}

	#[test]
	fn iter_recent_mut() {
		let mut queue = Queue::new([]);
		queue.enqueue((
			Skill {
				name: "a".to_owned(),

				..default()
			},
			SlotKey::BottomHand(Side::Right),
		));
		queue.enqueue((
			Skill {
				name: "b".to_owned(),
				..default()
			},
			SlotKey::BottomHand(Side::Left),
		));

		assert_eq!(
			(
				false,
				vec![
					&mut QueuedSkill {
						slot_key: SlotKey::BottomHand(Side::Right),
						skill: Skill {
							name: "a".to_owned(),
							..default()
						},
						..default()
					},
					&mut QueuedSkill {
						slot_key: SlotKey::BottomHand(Side::Left),
						skill: Skill {
							name: "b".to_owned(),
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
			slot_key: SlotKey::BottomHand(Side::Right),
			skill: Skill {
				name: "a".to_owned(),
				..default()
			},
			..default()
		}]);
		queue.enqueue((
			Skill {
				name: "b".to_owned(),

				..default()
			},
			SlotKey::BottomHand(Side::Right),
		));
		queue.enqueue((
			Skill {
				name: "c".to_owned(),

				..default()
			},
			SlotKey::BottomHand(Side::Left),
		));

		assert_eq!(
			(
				false,
				vec![
					&mut QueuedSkill {
						slot_key: SlotKey::BottomHand(Side::Right),
						skill: Skill {
							name: "b".to_owned(),
							..default()
						},
						..default()
					},
					&mut QueuedSkill {
						slot_key: SlotKey::BottomHand(Side::Left),
						skill: Skill {
							name: "c".to_owned(),
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
			slot_key: SlotKey::BottomHand(Side::Right),
			skill: Skill {
				name: "a".to_owned(),
				..default()
			},
			..default()
		}]);
		queue.enqueue((
			Skill {
				name: "b".to_owned(),

				..default()
			},
			SlotKey::BottomHand(Side::Right),
		));
		queue.enqueue((
			Skill {
				name: "c".to_owned(),

				..default()
			},
			SlotKey::BottomHand(Side::Left),
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
			slot_key: SlotKey::BottomHand(Side::Right),
			skill: Skill {
				name: "a".to_owned(),
				..default()
			},
			..default()
		}]);
		queue.enqueue((
			Skill {
				name: "b".to_owned(),

				..default()
			},
			SlotKey::BottomHand(Side::Right),
		));
		queue.enqueue((
			Skill {
				name: "c".to_owned(),

				..default()
			},
			SlotKey::BottomHand(Side::Left),
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
	) -> Option<impl GetSkillBehavior + GetAnimation + StateDuration<SkillState>> {
		let skill = self.queue.front_mut()?;

		if self.duration.is_none() {
			self.duration = Some(Duration::default());
		}

		Some(ActiveSkill {
			skill: &skill.skill,
			slot_key: &skill.slot_key,
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

impl GetAnimation for ActiveSkill<'_> {
	fn animate(&self) -> Animate<Animation> {
		match (&self.skill.animate, self.slot_key) {
			(Animate::None, ..) => Animate::None,
			(Animate::Ignore, ..) => Animate::Ignore,
			(Animate::Some(a), SlotKey::TopHand(Side::Right)) => {
				Animate::Some(a.top_hand_right.clone())
			}
			(Animate::Some(a), SlotKey::TopHand(Side::Left)) => {
				Animate::Some(a.top_hand_left.clone())
			}
			(Animate::Some(a), SlotKey::BottomHand(Side::Right)) => {
				Animate::Some(a.btm_hand_right.clone())
			}
			(Animate::Some(a), SlotKey::BottomHand(Side::Left)) => {
				Animate::Some(a.btm_hand_left.clone())
			}
		}
	}
}

#[cfg(test)]
mod test_queue_active_skill {
	use super::*;
	use crate::{
		behaviors::{
			build_skill_shape::{BuildSkillShape, OnSkillStop},
			SkillBehaviorConfig,
		},
		skills::{Animate, RunSkillBehavior, SkillAnimation},
		traits::skill_builder::SkillShape,
	};
	use bevy::prelude::default;
	use common::traits::{animation::PlayMode, load_asset::Path};

	#[test]
	fn get_phasing_times_waiting() {
		let mut queue = Queue {
			queue: VecDeque::from([
				QueuedSkill {
					skill: Skill {
						cast_time: Duration::from_millis(1),
						..default()
					},
					slot_key: SlotKey::BottomHand(Side::Right),
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
					slot_key: SlotKey::BottomHand(Side::Right),
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
					slot_key: SlotKey::BottomHand(Side::Right),
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
				slot_key: SlotKey::BottomHand(Side::Right),
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
				slot_key: SlotKey::BottomHand(Side::Right),
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
				slot_key: SlotKey::BottomHand(Side::Right),
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
				slot_key: SlotKey::BottomHand(Side::Right),
				..default()
			}]),
			..default()
		};

		queue.flush();

		assert_eq!(
			VecDeque::from([QueuedSkill {
				slot_key: SlotKey::BottomHand(Side::Right),
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
				slot_key: SlotKey::BottomHand(Side::Right),
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
				contact: commands.spawn_empty().id(),
				projection: commands.spawn_empty().id(),
				on_skill_stop: OnSkillStop::Ignore,
			}));

		let active = ActiveSkill {
			skill: &Skill {
				behavior: RunSkillBehavior::OnActive(behaviors.clone()),
				..default()
			},
			slot_key: &SlotKey::BottomHand(Side::Left),
			mode: &mut Activation::default(),
			duration: &mut Duration::default(),
		};

		assert_eq!(
			(
				SlotKey::BottomHand(Side::Left),
				RunSkillBehavior::OnActive(behaviors)
			),
			active.behavior()
		);
	}

	#[test]
	fn test_start_behavior_fn_on_aim() {
		let behaviors =
			SkillBehaviorConfig::from_shape(BuildSkillShape::Fn(|commands, _, _, _| SkillShape {
				contact: commands.spawn_empty().id(),
				projection: commands.spawn_empty().id(),
				on_skill_stop: OnSkillStop::Ignore,
			}));

		let active = ActiveSkill {
			skill: &mut Skill {
				behavior: RunSkillBehavior::OnAim(behaviors.clone()),
				..default()
			},
			slot_key: &SlotKey::TopHand(Side::Right),
			mode: &mut Activation::default(),
			duration: &mut Duration::default(),
		};

		assert_eq!(
			(
				SlotKey::TopHand(Side::Right),
				RunSkillBehavior::OnAim(behaviors)
			),
			active.behavior()
		);
	}

	#[test]
	fn get_animations() {
		let animation = SkillAnimation {
			top_hand_left: Animation::new(Path::from("path/top/left"), PlayMode::Repeat),
			top_hand_right: Animation::new(Path::from("path/top/right"), PlayMode::Replay),
			btm_hand_left: Animation::new(Path::from("path/bottom/left"), PlayMode::Repeat),
			btm_hand_right: Animation::new(Path::from("path/bottom/right"), PlayMode::Replay),
		};

		let actives = [
			ActiveSkill {
				skill: &Skill {
					animate: Animate::Some(animation.clone()),
					..default()
				},
				slot_key: &SlotKey::BottomHand(Side::Right),
				mode: &mut Activation::default(),
				duration: &mut Duration::default(),
			},
			ActiveSkill {
				skill: &Skill {
					animate: Animate::Some(animation.clone()),
					..default()
				},
				slot_key: &SlotKey::BottomHand(Side::Left),
				mode: &mut Activation::default(),
				duration: &mut Duration::default(),
			},
		];

		assert_eq!(
			[
				Animate::Some(animation.btm_hand_right),
				Animate::Some(animation.btm_hand_left),
			],
			actives.map(|s| s.animate()),
		)
	}

	#[test]
	fn get_ignore_animation() {
		let active = ActiveSkill {
			skill: &mut Skill {
				animate: Animate::Ignore,
				..default()
			},
			slot_key: &SlotKey::BottomHand(Side::Right),
			mode: &mut Activation::default(),
			duration: &mut Duration::default(),
		};

		assert_eq!(Animate::Ignore, active.animate())
	}

	#[test]
	fn get_none_animation() {
		let active = ActiveSkill {
			skill: &Skill {
				animate: Animate::None,
				..default()
			},
			slot_key: &SlotKey::BottomHand(Side::Right),
			mode: &mut Activation::default(),
			duration: &mut Duration::default(),
		};

		assert_eq!(Animate::None, active.animate())
	}
}
