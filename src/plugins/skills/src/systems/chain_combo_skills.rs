use std::collections::HashMap;

use crate::{
	components::{
		queue::{DequeueAble, Queue, QueueCollection},
		ComboTreeRunning,
		ComboTreeTemplate,
		SlotKey,
	},
	skill::{Queued, Skill, SkillComboTree},
	traits::{ComboNext, GetOldLastMut, IterRecentMut},
};
use bevy::{
	ecs::{
		entity::Entity,
		system::{Commands, Query},
	},
	utils::default,
};
use common::traits::{try_insert_on::TryInsertOn, try_remove_from::TryRemoveFrom};

type ComboComponents<'a, TNext, TEnqueue> = (
	Entity,
	&'a mut Queue<TEnqueue, QueueCollection<DequeueAble>>,
	&'a ComboTreeTemplate<TNext>,
	Option<&'a ComboTreeRunning<TNext>>,
);

pub(crate) fn chain_combo_skills<
	TNext: Clone + ComboNext + Send + Sync + 'static,
	TEnqueue: GetOldLastMut<Skill<Queued>> + IterRecentMut<Skill<Queued>> + Send + Sync + 'static,
>(
	mut commands: Commands,
	mut agents: Query<ComboComponents<TNext, TEnqueue>>,
) {
	for (id, mut queue, template, running_template) in &mut agents {
		let Queue::Enqueue(enqueue) = queue.as_mut() else {
			continue;
		};
		let Some(mut trigger_skill) = get_trigger_skill(enqueue) else {
			commands.try_remove_from::<ComboTreeRunning<TNext>>(id);
			continue;
		};

		let mut running_combos = running_template.map(|r| r.0.clone()).unwrap_or_default();
		let mut recently_queued_skills = enqueue.iter_recent_mut();
		let mut apply_skill_combos = || {
			let triggers = match running_combos.is_empty() {
				true => &template.0,
				false => &running_combos,
			};

			let Some(combo_skill) = recently_queued_skills.next() else {
				return false;
			};

			let Some(trigger) = triggers.get(&trigger_skill.data.0) else {
				running_combos = default();
				return false;
			};

			if trigger.skill.name != trigger_skill.name {
				running_combos = default();
				return false;
			}

			let Some((key, combo)) = get_combo(trigger, &trigger_skill, combo_skill) else {
				running_combos = default();
				return false;
			};

			*combo_skill = combo.skill.clone().with(combo_skill.data.clone());

			running_combos = HashMap::from([(key, combo.clone())]);
			trigger_skill = combo_skill.clone();

			true
		};

		// would prefer a recursive call, but can't call lambda from within the same lambda,
		// so we iterate and mutate the would be call arguments.
		while apply_skill_combos() {}

		if running_combos.is_empty() {
			commands.try_remove_from::<ComboTreeRunning<TNext>>(id);
		} else {
			commands.try_insert_on(id, ComboTreeRunning(running_combos));
		}
	}
}

fn get_trigger_skill<TEnqueue: GetOldLastMut<Skill<Queued>>>(
	enqueue: &mut TEnqueue,
) -> Option<Skill<Queued>> {
	enqueue.get_old_last_mut().cloned()
}

fn get_combo<TNext: Clone + ComboNext + Send + Sync + 'static>(
	trigger: &SkillComboTree<TNext>,
	trigger_skill: &Skill<Queued>,
	combo_skill: &Skill<Queued>,
) -> Option<(SlotKey, SkillComboTree<TNext>)> {
	trigger
		.next
		.to_vec(trigger_skill)
		.into_iter()
		.find(|(k, _)| k == &combo_skill.data.0)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{components::SlotKey, skill::SkillComboTree};
	use bevy::{
		app::{App, Update},
		utils::default,
	};
	use common::{components::Side, test_tools::utils::SingleThreadedApp};
	use mockall::{mock, predicate::eq};
	use std::collections::HashMap;

	#[derive(Debug, PartialEq)]
	struct _Enqueue {
		added_last_frame: Option<Skill<Queued>>,
		added_this_frame: Vec<Skill<Queued>>,
	}

	impl GetOldLastMut<Skill<Queued>> for _Enqueue {
		fn get_old_last_mut<'a>(&'a mut self) -> Option<&'a mut Skill<Queued>>
		where
			Skill<Queued>: 'a,
		{
			self.added_last_frame.as_mut()
		}
	}

	impl IterRecentMut<Skill<Queued>> for _Enqueue {
		fn iter_recent_mut<'a>(
			&'a mut self,
		) -> impl DoubleEndedIterator<Item = &'a mut Skill<Queued>>
		where
			Skill<Queued>: 'a,
		{
			self.added_this_frame.iter_mut()
		}
	}

	#[derive(Clone, Debug, PartialEq)]
	struct _Next(Vec<(SlotKey, SkillComboTree<Self>)>);

	impl ComboNext for _Next {
		fn to_vec(&self, _skill: &Skill<Queued>) -> Vec<(SlotKey, SkillComboTree<Self>)> {
			self.0.clone()
		}
	}

	mock! {
		_Next{}
		impl ComboNext for _Next {
			fn to_vec(&self, _skill: &Skill<Queued>) -> Vec<(SlotKey, SkillComboTree<Self>)> {}
		}
		impl Clone for _Next {
			fn clone(&self) -> Self {}
		}
	}

	fn empty_mock() -> Mock_Next {
		let mut fake_clone = Mock_Next::default();
		fake_clone.expect_clone().returning(empty_mock);
		fake_clone.expect_to_vec().return_const(vec![]);
		fake_clone
	}

	fn setup<TNext: ComboNext + Clone + Send + Sync + 'static, const N: usize>(
		combos_template: [(SlotKey, SkillComboTree<TNext>); N],
		queue: Queue<_Enqueue, QueueCollection<DequeueAble>>,
	) -> (App, Entity) {
		let mut app = App::new_single_threaded([Update]);
		app.add_systems(Update, chain_combo_skills::<TNext, _Enqueue>);

		let agent = app
			.world
			.spawn((ComboTreeTemplate(HashMap::from(combos_template)), queue))
			.id();

		(app, agent)
	}

	#[test]
	fn set_set_combo_from_template() {
		let (mut app, agent) = setup(
			[(
				SlotKey::Hand(Side::Main),
				SkillComboTree {
					skill: Skill {
						name: "trigger a",
						..default()
					},
					next: _Next(vec![(
						SlotKey::Hand(Side::Main),
						SkillComboTree {
							skill: Skill {
								name: "combo a",
								..default()
							},
							next: _Next(vec![]),
						},
					)]),
				},
			)],
			Queue::Enqueue(_Enqueue {
				added_last_frame: Some(Skill {
					name: "trigger a",
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}),
				added_this_frame: vec![Skill {
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}],
			}),
		);

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			&Queue::Enqueue(_Enqueue {
				added_last_frame: Some(Skill {
					name: "trigger a",
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}),
				added_this_frame: vec![Skill {
					name: "combo a",
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}],
			}),
			agent
				.get::<Queue<_Enqueue, QueueCollection<DequeueAble>>>()
				.unwrap()
		)
	}

	#[test]
	fn do_not_set_set_combo_from_template_when_trigger_key_mismatch() {
		let (mut app, agent) = setup(
			[(
				SlotKey::Hand(Side::Off),
				SkillComboTree {
					skill: Skill {
						name: "trigger a",
						..default()
					},
					next: _Next(vec![(
						SlotKey::Hand(Side::Main),
						SkillComboTree {
							skill: Skill {
								name: "combo a",
								..default()
							},
							next: _Next(vec![]),
						},
					)]),
				},
			)],
			Queue::Enqueue(_Enqueue {
				added_last_frame: Some(Skill {
					name: "trigger a",
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}),
				added_this_frame: vec![Skill {
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}],
			}),
		);

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			&Queue::Enqueue(_Enqueue {
				added_last_frame: Some(Skill {
					name: "trigger a",
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}),
				added_this_frame: vec![Skill {
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}],
			}),
			agent
				.get::<Queue<_Enqueue, QueueCollection<DequeueAble>>>()
				.unwrap()
		)
	}

	#[test]
	fn do_not_set_set_combo_from_template_when_trigger_name_mismatch() {
		let (mut app, agent) = setup(
			[(
				SlotKey::Hand(Side::Main),
				SkillComboTree {
					skill: Skill {
						name: "trigger b",
						..default()
					},
					next: _Next(vec![(
						SlotKey::Hand(Side::Main),
						SkillComboTree {
							skill: Skill {
								name: "combo a",
								..default()
							},
							next: _Next(vec![]),
						},
					)]),
				},
			)],
			Queue::Enqueue(_Enqueue {
				added_last_frame: Some(Skill {
					name: "trigger a",
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}),
				added_this_frame: vec![Skill {
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}],
			}),
		);

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			&Queue::Enqueue(_Enqueue {
				added_last_frame: Some(Skill {
					name: "trigger a",
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}),
				added_this_frame: vec![Skill {
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}],
			}),
			agent
				.get::<Queue<_Enqueue, QueueCollection<DequeueAble>>>()
				.unwrap()
		)
	}

	#[test]
	fn do_not_set_set_combo_from_template_when_combo_key_mismatch() {
		let (mut app, agent) = setup(
			[(
				SlotKey::Hand(Side::Main),
				SkillComboTree {
					skill: Skill {
						name: "trigger a",
						..default()
					},
					next: _Next(vec![(
						SlotKey::Hand(Side::Off),
						SkillComboTree {
							skill: Skill {
								name: "combo a",
								..default()
							},
							next: _Next(vec![]),
						},
					)]),
				},
			)],
			Queue::Enqueue(_Enqueue {
				added_last_frame: Some(Skill {
					name: "trigger a",
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}),
				added_this_frame: vec![Skill {
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}],
			}),
		);

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			&Queue::Enqueue(_Enqueue {
				added_last_frame: Some(Skill {
					name: "trigger a",
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}),
				added_this_frame: vec![Skill {
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}],
			}),
			agent
				.get::<Queue<_Enqueue, QueueCollection<DequeueAble>>>()
				.unwrap()
		)
	}

	#[test]
	fn call_next_to_vec_with_combo_skill_candidate() {
		let mut next = Mock_Next::default();
		next.expect_to_vec()
			.times(1)
			.with(eq(Skill {
				name: "trigger a",
				data: Queued(SlotKey::Hand(Side::Main)),
				..default()
			}))
			.return_const(vec![]);
		next.expect_clone().returning(empty_mock);

		let (mut app, ..) = setup(
			[(
				SlotKey::Hand(Side::Main),
				SkillComboTree {
					skill: Skill {
						name: "trigger a",
						..default()
					},
					next,
				},
			)],
			Queue::Enqueue(_Enqueue {
				added_last_frame: Some(Skill {
					name: "trigger a",
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}),
				added_this_frame: vec![Skill {
					name: "other",
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}],
			}),
		);

		app.update();
	}

	#[test]
	fn set_running_combo_template_from_template_subtree() {
		let (mut app, agent) = setup(
			[(
				SlotKey::Hand(Side::Main),
				SkillComboTree {
					skill: Skill {
						name: "trigger a",
						..default()
					},
					next: _Next(vec![(
						SlotKey::Hand(Side::Main),
						SkillComboTree {
							skill: Skill {
								name: "combo a",
								..default()
							},
							next: _Next(vec![(
								SlotKey::Hand(Side::Off),
								SkillComboTree {
									skill: Skill {
										name: "combo b",
										..default()
									},
									next: _Next(vec![]),
								},
							)]),
						},
					)]),
				},
			)],
			Queue::Enqueue(_Enqueue {
				added_last_frame: Some(Skill {
					name: "trigger a",
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}),
				added_this_frame: vec![Skill {
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}],
			}),
		);

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&ComboTreeRunning(HashMap::from([(
				SlotKey::Hand(Side::Main),
				SkillComboTree {
					skill: Skill {
						name: "combo a",
						..default()
					},
					next: _Next(vec![(
						SlotKey::Hand(Side::Off),
						SkillComboTree {
							skill: Skill {
								name: "combo b",
								..default()
							},
							next: _Next(vec![]),
						},
					)]),
				},
			)]))),
			agent.get::<ComboTreeRunning<_Next>>()
		)
	}

	#[test]
	fn use_running_combo_template() {
		let (mut app, agent) = setup(
			[(
				SlotKey::Hand(Side::Main),
				SkillComboTree {
					skill: Skill {
						name: "trigger a",
						..default()
					},
					next: _Next(vec![(
						SlotKey::Hand(Side::Main),
						SkillComboTree {
							skill: Skill {
								name: "do not use this combo",
								..default()
							},
							next: _Next(vec![]),
						},
					)]),
				},
			)],
			Queue::Enqueue(_Enqueue {
				added_last_frame: Some(Skill {
					name: "trigger a",
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}),
				added_this_frame: vec![Skill {
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}],
			}),
		);

		app.world
			.entity_mut(agent)
			.insert(ComboTreeRunning(HashMap::from([(
				SlotKey::Hand(Side::Main),
				SkillComboTree {
					skill: Skill {
						name: "trigger a",
						..default()
					},
					next: _Next(vec![(
						SlotKey::Hand(Side::Main),
						SkillComboTree {
							skill: Skill {
								name: "combo a",
								..default()
							},
							next: _Next(vec![]),
						},
					)]),
				},
			)])));

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			&Queue::Enqueue(_Enqueue {
				added_last_frame: Some(Skill {
					name: "trigger a",
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}),
				added_this_frame: vec![Skill {
					name: "combo a",
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}],
			}),
			agent
				.get::<Queue<_Enqueue, QueueCollection<DequeueAble>>>()
				.unwrap()
		)
	}

	#[test]
	fn ignore_running_combo_template_if_empty() {
		let (mut app, agent) = setup(
			[(
				SlotKey::Hand(Side::Main),
				SkillComboTree {
					skill: Skill {
						name: "trigger a",
						..default()
					},
					next: _Next(vec![(
						SlotKey::Hand(Side::Main),
						SkillComboTree {
							skill: Skill {
								name: "combo a",
								..default()
							},
							next: _Next(vec![]),
						},
					)]),
				},
			)],
			Queue::Enqueue(_Enqueue {
				added_last_frame: Some(Skill {
					name: "trigger a",
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}),
				added_this_frame: vec![Skill {
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}],
			}),
		);

		app.world
			.entity_mut(agent)
			.insert(ComboTreeRunning::<_Next>(HashMap::from([])));

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			&Queue::Enqueue(_Enqueue {
				added_last_frame: Some(Skill {
					name: "trigger a",
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}),
				added_this_frame: vec![Skill {
					name: "combo a",
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}],
			}),
			agent
				.get::<Queue<_Enqueue, QueueCollection<DequeueAble>>>()
				.unwrap()
		)
	}

	#[test]
	fn remove_running_combo_when_no_last_skill() {
		let (mut app, agent) = setup(
			[(
				SlotKey::Hand(Side::Main),
				SkillComboTree {
					skill: Skill {
						name: "trigger a",
						..default()
					},
					next: _Next(vec![(
						SlotKey::Hand(Side::Main),
						SkillComboTree {
							skill: Skill {
								name: "combo a",
								..default()
							},
							next: _Next(vec![]),
						},
					)]),
				},
			)],
			Queue::Enqueue(_Enqueue {
				added_last_frame: None,
				added_this_frame: vec![Skill {
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}],
			}),
		);

		app.world
			.entity_mut(agent)
			.insert(ComboTreeRunning::<_Next>(default()));

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(None, agent.get::<ComboTreeRunning<_Next>>())
	}

	#[test]
	fn remove_running_combo_when_trigger_key_mismatch() {
		let (mut app, agent) = setup(
			[(
				SlotKey::Hand(Side::Off),
				SkillComboTree {
					skill: Skill {
						name: "trigger a",
						..default()
					},
					next: _Next(vec![(
						SlotKey::Hand(Side::Main),
						SkillComboTree {
							skill: Skill {
								name: "combo a",
								..default()
							},
							next: _Next(vec![]),
						},
					)]),
				},
			)],
			Queue::Enqueue(_Enqueue {
				added_last_frame: Some(Skill {
					name: "trigger a",
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}),
				added_this_frame: vec![Skill {
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}],
			}),
		);

		app.world
			.entity_mut(agent)
			.insert(ComboTreeRunning::<_Next>(default()));

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(None, agent.get::<ComboTreeRunning<_Next>>())
	}

	#[test]
	fn remove_running_combo_when_trigger_name_mismatch() {
		let (mut app, agent) = setup(
			[(
				SlotKey::Hand(Side::Main),
				SkillComboTree {
					skill: Skill {
						name: "wrong trigger name",
						..default()
					},
					next: _Next(vec![(
						SlotKey::Hand(Side::Main),
						SkillComboTree {
							skill: Skill {
								name: "combo a",
								..default()
							},
							next: _Next(vec![]),
						},
					)]),
				},
			)],
			Queue::Enqueue(_Enqueue {
				added_last_frame: Some(Skill {
					name: "trigger a",
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}),
				added_this_frame: vec![Skill {
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}],
			}),
		);

		app.world
			.entity_mut(agent)
			.insert(ComboTreeRunning::<_Next>(default()));

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(None, agent.get::<ComboTreeRunning<_Next>>())
	}

	#[test]
	fn remove_running_combo_when_combo_key_mismatch() {
		let (mut app, agent) = setup(
			[(
				SlotKey::Hand(Side::Main),
				SkillComboTree {
					skill: Skill {
						name: "trigger a",
						..default()
					},
					next: _Next(vec![(
						SlotKey::Hand(Side::Off),
						SkillComboTree {
							skill: Skill {
								name: "combo a",
								..default()
							},
							next: _Next(vec![]),
						},
					)]),
				},
			)],
			Queue::Enqueue(_Enqueue {
				added_last_frame: Some(Skill {
					name: "trigger a",
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}),
				added_this_frame: vec![Skill {
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}],
			}),
		);

		app.world
			.entity_mut(agent)
			.insert(ComboTreeRunning::<_Next>(default()));

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(None, agent.get::<ComboTreeRunning<_Next>>())
	}

	#[test]
	fn set_set_combo_from_template_for_combo_chain() {
		let (mut app, agent) = setup(
			[(
				SlotKey::Hand(Side::Main),
				SkillComboTree {
					skill: Skill {
						name: "trigger a",
						..default()
					},
					next: _Next(vec![(
						SlotKey::Hand(Side::Main),
						SkillComboTree {
							skill: Skill {
								name: "combo a",
								..default()
							},
							next: _Next(vec![(
								SlotKey::Hand(Side::Main),
								SkillComboTree {
									skill: Skill {
										name: "combo b",
										..default()
									},
									next: _Next(vec![]),
								},
							)]),
						},
					)]),
				},
			)],
			Queue::Enqueue(_Enqueue {
				added_last_frame: Some(Skill {
					name: "trigger a",
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}),
				added_this_frame: vec![
					Skill {
						data: Queued(SlotKey::Hand(Side::Main)),
						..default()
					},
					Skill {
						data: Queued(SlotKey::Hand(Side::Main)),
						..default()
					},
				],
			}),
		);

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			&Queue::Enqueue(_Enqueue {
				added_last_frame: Some(Skill {
					name: "trigger a",
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}),
				added_this_frame: vec![
					Skill {
						name: "combo a",
						data: Queued(SlotKey::Hand(Side::Main)),
						..default()
					},
					Skill {
						name: "combo b",
						data: Queued(SlotKey::Hand(Side::Main)),
						..default()
					}
				],
			}),
			agent
				.get::<Queue<_Enqueue, QueueCollection<DequeueAble>>>()
				.unwrap()
		)
	}
}
