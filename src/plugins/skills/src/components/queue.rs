use super::SlotKey;
use crate::{
	skills::{Activation, Animate, Queued, Skill, SkillState, StartBehaviorFn, StopBehaviorFn},
	traits::{
		Enqueue,
		Execution,
		Flush,
		GetActiveSkill,
		GetAnimation,
		Iter,
		IterAddedMut,
		IterMutWithKeys,
	},
};
use animations::animation::Animation;
use bevy::{
	ecs::{component::Component, entity::Entity, system::Commands},
	hierarchy::DespawnRecursiveExt,
	utils::default,
};
use common::{components::Side, traits::state_duration::StateDuration};
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

impl Enqueue<(Skill, SlotKey)> for Queue {
	fn enqueue(&mut self, item: (Skill, SlotKey)) {
		if self.state == State::Flushed {
			let len_before_change = self.queue.len();
			self.state = State::Changed { len_before_change }
		}

		let (skill, slot_key) = item;

		self.queue.push_back(skill.with(Queued {
			slot_key,
			..default()
		}));
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

impl IterMutWithKeys<SlotKey, Skill<Queued>> for Queue {
	fn iter_mut_with_keys<'a>(
		&'a mut self,
	) -> impl DoubleEndedIterator<Item = (SlotKey, &'a mut Skill<Queued>)>
	where
		Skill<Queued>: 'a,
	{
		self.queue.iter_mut().map(|s| (s.data.slot_key, s))
	}
}

impl IterAddedMut<Skill<Queued>> for Queue {
	fn iter_added_mut<'a>(&'a mut self) -> impl DoubleEndedIterator<Item = &'a mut Skill<Queued>>
	where
		Skill<Queued>: 'a,
	{
		let unchanged_len = match self.state {
			State::Flushed => self.queue.len(),
			State::Changed { len_before_change } => len_before_change,
		};

		self.queue.iter_mut().skip(unchanged_len)
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
		queue.enqueue((
			Skill {
				name: "my skill",
				..default()
			},
			SlotKey::Hand(Side::Main),
		));

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
		queue.enqueue((
			Skill {
				name: "skill a",
				..default()
			},
			SlotKey::Hand(Side::Off),
		));
		queue.enqueue((
			Skill {
				name: "skill b",
				..default()
			},
			SlotKey::Hand(Side::Main),
		));

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
	fn flush_with_one_skill() {
		let mut queue = Queue::new([Skill {
			name: "my skill",
			data: Queued {
				slot_key: SlotKey::Hand(Side::Main),
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
			(queue_after_1_flush, queue_after_2_flushes)
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
	fn iter_mut_with_keys() {
		let mut queue = Queue::new([]);
		queue.enqueue((
			Skill {
				name: "skill a",
				..default()
			},
			SlotKey::Hand(Side::Off),
		));
		queue.enqueue((
			Skill {
				name: "skill b",
				..default()
			},
			SlotKey::Hand(Side::Main),
		));

		assert_eq!(
			vec![
				(
					SlotKey::Hand(Side::Off),
					&mut Skill {
						name: "skill a",
						data: Queued {
							slot_key: SlotKey::Hand(Side::Off),
							..default()
						},
						..default()
					}
				),
				(
					SlotKey::Hand(Side::Main),
					&mut Skill {
						name: "skill b",
						data: Queued {
							slot_key: SlotKey::Hand(Side::Main),
							..default()
						},
						..default()
					}
				)
			],
			queue.iter_mut_with_keys().collect::<Vec<_>>()
		)
	}

	#[test]
	fn iter_recent_mut() {
		let mut queue = Queue::new([]);
		queue.enqueue((
			Skill {
				name: "a",

				..default()
			},
			SlotKey::Hand(Side::Main),
		));
		queue.enqueue((
			Skill {
				name: "b",
				..default()
			},
			SlotKey::Hand(Side::Off),
		));

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
			queue.iter_added_mut().collect::<Vec<_>>()
		)
	}

	#[test]
	fn iter_recent_mut_only_new() {
		let mut queue = Queue::new([Skill {
			name: "a",
			data: Queued {
				slot_key: SlotKey::Hand(Side::Main),
				..default()
			},
			..default()
		}]);
		queue.enqueue((
			Skill {
				name: "b",

				..default()
			},
			SlotKey::Hand(Side::Main),
		));
		queue.enqueue((
			Skill {
				name: "c",

				..default()
			},
			SlotKey::Hand(Side::Off),
		));

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
			queue.iter_added_mut().collect::<Vec<_>>()
		)
	}

	#[test]
	fn iter_recent_mut_empty_after_flush() {
		let mut queue = Queue::new([Skill {
			name: "a",
			data: Queued {
				slot_key: SlotKey::Hand(Side::Main),
				..default()
			},
			..default()
		}]);
		queue.enqueue((
			Skill {
				name: "b",

				..default()
			},
			SlotKey::Hand(Side::Main),
		));
		queue.enqueue((
			Skill {
				name: "c",

				..default()
			},
			SlotKey::Hand(Side::Off),
		));

		queue.flush();

		assert_eq!(
			vec![] as Vec<&mut Skill<Queued>>,
			queue.iter_added_mut().collect::<Vec<_>>()
		)
	}

	#[test]
	fn iter_recent_mut_empty_after_flush_with_active_duration() {
		let mut queue = Queue::new([Skill {
			name: "a",
			data: Queued {
				slot_key: SlotKey::Hand(Side::Main),
				..default()
			},
			..default()
		}]);
		queue.enqueue((
			Skill {
				name: "b",

				..default()
			},
			SlotKey::Hand(Side::Main),
		));
		queue.enqueue((
			Skill {
				name: "c",

				..default()
			},
			SlotKey::Hand(Side::Off),
		));

		queue.duration = Some(Duration::from_millis(42));
		queue.flush();

		assert_eq!(
			vec![] as Vec<&mut Skill<Queued>>,
			queue.iter_added_mut().collect::<Vec<_>>()
		)
	}
}

struct ActiveSkill<'a> {
	duration: &'a mut Duration,
	skill: &'a mut Skill<Queued>,
}

impl GetActiveSkill<Animation, SkillState> for Queue {
	fn get_active(
		&mut self,
	) -> Option<impl Execution + GetAnimation<Animation> + StateDuration<SkillState>> {
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
			(SkillState::Aim, Activation::Primed | Activation::Waiting) => Duration::MAX,
			(SkillState::Aim, Activation::ActiveAfter(duration)) => *duration,
			(SkillState::Active, _) => self.skill.active,
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
		if !self.skill.execution.execution_stop_on_skill_stop {
			return None;
		}

		Some(remove_entity_recursive)
	}
}

fn remove_entity_recursive(commands: &mut Commands, entity: Entity) {
	let Some(entity) = commands.get_entity(entity) else {
		return;
	};
	entity.despawn_recursive();
}

impl<'a> GetAnimation<Animation> for ActiveSkill<'a> {
	fn animate(&self) -> Animate<Animation> {
		match (&self.skill.animate, self.skill.data.slot_key) {
			(Animate::None, ..) => Animate::None,
			(Animate::Ignore, ..) => Animate::Ignore,
			(Animate::Some(a), SlotKey::Hand(Side::Main)) => Animate::Some(a.right.clone()),
			(Animate::Some(a), SlotKey::Hand(Side::Off)) => Animate::Some(a.left.clone()),
		}
	}
}

#[cfg(test)]
mod test_queue_active_skill {
	use super::*;
	use crate::{
		components::SlotKey,
		skills::{Animate, SkillAnimation, SkillCaster, SkillExecution, SkillSpawner, Target},
	};
	use animations::animation::PlayMode;
	use bevy::{
		app::{App, Update},
		hierarchy::BuildWorldChildren,
		prelude::default,
	};
	use common::{components::Side, traits::load_asset::Path};

	#[test]
	fn get_phasing_times_waiting() {
		let mut queue = Queue {
			queue: VecDeque::from([
				Skill {
					data: Queued {
						slot_key: SlotKey::Hand(Side::Main),
						mode: Activation::Waiting,
					},
					active: Duration::from_millis(1),
					..default()
				},
				Skill::default(),
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
				Skill {
					data: Queued {
						slot_key: SlotKey::Hand(Side::Main),
						mode: Activation::Primed,
					},
					active: Duration::from_millis(1),
					..default()
				},
				Skill::default(),
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
				Skill {
					data: Queued {
						slot_key: SlotKey::Hand(Side::Main),
						mode: Activation::ActiveAfter(Duration::from_millis(42)),
					},
					active: Duration::from_millis(1),
					..default()
				},
				Skill::default(),
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
	fn do_not_pop_front_on_flush_when_duration_set() {
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

		queue.flush();

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
		fn run(_: &mut Commands, _: &SkillCaster, _: &SkillSpawner, _: &Target) -> Entity {
			Entity::from_raw(100)
		}

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
	fn test_stop_behavior_fn_when_set_to_stoppable() {
		let active = ActiveSkill {
			skill: &mut Skill {
				data: Queued {
					slot_key: SlotKey::Hand(Side::Main),
					..default()
				},
				execution: SkillExecution {
					execution_stop_on_skill_stop: true,
					..default()
				},
				..default()
			},
			duration: &mut Duration::default(),
		};

		assert_eq!(
			Some(remove_entity_recursive as usize),
			active.get_stop().map(|f| f as usize)
		);
	}

	#[test]
	fn test_stop_behavior_fn_when_not_set_to_stoppable() {
		let active = ActiveSkill {
			skill: &mut Skill {
				data: Queued {
					slot_key: SlotKey::Hand(Side::Main),
					..default()
				},
				execution: SkillExecution {
					execution_stop_on_skill_stop: false,
					..default()
				},
				..default()
			},
			duration: &mut Duration::default(),
		};

		assert_eq!(None, active.get_stop().map(|f| f as usize));
	}

	#[test]
	fn test_stop_fn() {
		let mut app = App::new();
		let entity = app.world.spawn_empty().id();
		app.world.spawn_empty().set_parent(entity);

		app.add_systems(Update, move |mut commands: Commands| {
			remove_entity_recursive(&mut commands, entity)
		});

		app.update();

		assert!(app.world.entities().is_empty());
	}

	#[test]
	fn get_animations() {
		let animation = SkillAnimation {
			left: Animation::new(Path::from("path/left"), PlayMode::Repeat),
			right: Animation::new(Path::from("path/right"), PlayMode::Replay),
		};

		let actives = [
			ActiveSkill {
				duration: &mut Duration::default(),
				skill: &mut Skill {
					data: Queued {
						slot_key: SlotKey::Hand(Side::Main),
						..default()
					},
					animate: Animate::Some(animation.clone()),
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
					animate: Animate::Some(animation.clone()),
					..default()
				},
			},
		];

		assert_eq!(
			[
				Animate::Some(animation.right),
				Animate::Some(animation.left),
			],
			actives.map(|s| s.animate()),
		)
	}

	#[test]
	fn get_ignore_animation() {
		let active = ActiveSkill {
			duration: &mut Duration::default(),
			skill: &mut Skill {
				data: Queued {
					slot_key: SlotKey::Hand(Side::Main),
					..default()
				},
				animate: Animate::Ignore,
				..default()
			},
		};

		assert_eq!(Animate::Ignore, active.animate())
	}

	#[test]
	fn get_none_animation() {
		let active = ActiveSkill {
			duration: &mut Duration::default(),
			skill: &mut Skill {
				data: Queued {
					slot_key: SlotKey::Hand(Side::Main),
					..default()
				},
				animate: Animate::None,
				..default()
			},
		};

		assert_eq!(Animate::None, active.animate())
	}
}
