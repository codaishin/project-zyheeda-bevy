use crate::{
	skills::{RunSkillBehavior, SkillState},
	traits::{Flush, GetActiveSkill, GetSkillBehavior, Schedule},
};
use bevy::{
	ecs::{component::Mutable, system::StaticSystemParam},
	prelude::*,
};
use common::{
	tools::action_key::slot::SlotKey,
	traits::{
		accessors::get::GetContextMut,
		handles_orientation::{Face, Facing, OverrideFace},
		state_duration::{StateMeta, UpdatedStates},
		thread_safe::ThreadSafe,
	},
};
use std::time::Duration;

#[derive(PartialEq)]
enum Advancement {
	Finished,
	InProcess,
}

pub(crate) fn advance_active_skill<TGetSkill, TFacing, TSkillExecutor, TTime>(
	time: Res<Time<TTime>>,
	mut agents: Query<(Entity, &mut TGetSkill, &mut TSkillExecutor)>,
	mut facing: StaticSystemParam<TFacing>,
) where
	TGetSkill: GetActiveSkill<SkillState> + Component<Mutability = Mutable>,
	TFacing: for<'c> GetContextMut<Facing, TContext<'c>: OverrideFace>,
	TSkillExecutor: Component<Mutability = Mutable> + Schedule<RunSkillBehavior> + Flush,
	TTime: Default + ThreadSafe,
{
	let delta = time.delta();

	for (entity, mut dequeue, skill_executer) in &mut agents {
		let Some(mut ctx) = TFacing::get_context_mut(&mut facing, Facing { entity }) else {
			continue;
		};
		let advancement = match dequeue.get_active() {
			Some(skill) => advance(skill, skill_executer, delta, &mut ctx),
			None => clear_side_effects(&mut ctx),
		};

		if advancement == Advancement::InProcess {
			continue;
		}

		dequeue.clear_active();
	}
}

fn clear_side_effects<TFacing>(facing: &mut TFacing) -> Advancement
where
	TFacing: OverrideFace,
{
	facing.stop_override_face();

	Advancement::InProcess
}

fn advance<TFacing, TSkillExecutor>(
	mut skill: impl GetSkillBehavior + UpdatedStates<SkillState>,
	mut skill_executer: Mut<TSkillExecutor>,
	delta: Duration,
	facing: &mut TFacing,
) -> Advancement
where
	TFacing: OverrideFace,
	TSkillExecutor: Schedule<RunSkillBehavior> + Flush,
{
	let skill = &mut skill;
	let states = skill.updated_states(delta);

	if states.contains(&StateMeta::Entering(SkillState::Aim)) {
		facing.override_face(Face::Target);
		schedule_start(&mut skill_executer, skill, run_on_aim);
	}

	if states.contains(&StateMeta::Entering(SkillState::Active)) {
		schedule_start(&mut skill_executer, skill, run_on_active);
	}

	if states.contains(&StateMeta::Done) {
		skill_executer.flush();
		return Advancement::Finished;
	}

	Advancement::InProcess
}

fn run_on_aim<TSkill>(skill: &TSkill) -> Option<(SlotKey, RunSkillBehavior)>
where
	TSkill: GetSkillBehavior,
{
	let (slot_key, behavior) = skill.behavior();
	match &behavior {
		RunSkillBehavior::OnAim(_) => Some((slot_key, behavior)),
		_ => None,
	}
}

fn run_on_active<TSkill>(skill: &TSkill) -> Option<(SlotKey, RunSkillBehavior)>
where
	TSkill: GetSkillBehavior,
{
	let (slot_key, behavior) = skill.behavior();
	match &behavior {
		RunSkillBehavior::OnActive(_) => Some((slot_key, behavior)),
		_ => None,
	}
}

fn schedule_start<TSkillExecutor, TSkill>(
	executer: &mut Mut<TSkillExecutor>,
	skill: &TSkill,
	get_start_fn: fn(&TSkill) -> Option<(SlotKey, RunSkillBehavior)>,
) where
	TSkillExecutor: Schedule<RunSkillBehavior>,
	TSkill: GetSkillBehavior,
{
	let Some((slot_key, start_fn)) = get_start_fn(skill) else {
		return;
	};
	executer.schedule(slot_key, start_fn);
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::{
		behaviors::{
			SkillBehaviorConfig,
			skill_shape::{OnSkillStop, SkillShape},
		},
		traits::skill_builder::SkillLayout,
	};
	use common::tools::action_key::slot::{PlayerSlot, Side};
	use macros::{NestedMocks, simple_mock};
	use mockall::{automock, mock, predicate::eq};
	use std::collections::HashSet;
	use testing::{IsChanged, MissingLastUpdate, Mock, NestedMocks, SingleThreadedApp, TickTime};

	#[derive(Component, Default)]
	struct _Dequeue {
		pub active: Option<Box<dyn FnMut() -> Mock_Skill + Sync + Send>>,
	}

	impl GetActiveSkill<SkillState> for _Dequeue {
		type TActive<'a>
			= Mock_Skill
		where
			Self: 'a;

		fn clear_active(&mut self) {
			self.active = None;
		}

		fn get_active(&mut self) -> Option<Self::TActive<'_>> {
			self.active.as_mut().map(|f| f())
		}
	}

	simple_mock! {
		_Skill {}
		impl UpdatedStates<SkillState> for _Skill {
			fn updated_states(&mut self, delta: Duration) -> HashSet<StateMeta<SkillState>>;
		}
		impl GetSkillBehavior for _Skill {
			fn behavior<'a>(&self) -> (SlotKey, RunSkillBehavior);
		}
	}

	#[derive(Component, NestedMocks)]
	struct _Facing {
		mock: Mock_Facing,
	}

	impl Default for _Facing {
		fn default() -> Self {
			Self::new().with_mock(|mock| {
				mock.expect_override_face().return_const(());
				mock.expect_stop_override_face().return_const(());
			})
		}
	}

	#[automock]
	impl OverrideFace for _Facing {
		fn override_face(&mut self, face: Face) {
			self.mock.override_face(face);
		}

		fn stop_override_face(&mut self) {
			self.mock.stop_override_face();
		}
	}

	#[derive(Component, NestedMocks)]
	struct _Executor {
		mock: Mock_Executor,
	}

	impl Schedule<RunSkillBehavior> for _Executor {
		fn schedule(&mut self, slot_key: SlotKey, start: RunSkillBehavior) {
			self.mock.schedule(slot_key, start)
		}
	}

	impl Flush for _Executor {
		fn flush(&mut self) {
			self.mock.flush()
		}
	}

	mock! {
		_Executor {}
		impl Schedule<RunSkillBehavior> for _Executor {
			fn schedule(&mut self, slot_key: SlotKey, start: RunSkillBehavior);
		}
		impl Flush for _Executor {
			fn flush(&mut self);
		}
	}

	fn skill_behavior(
		activation_type: impl Fn(SkillBehaviorConfig) -> RunSkillBehavior,
	) -> RunSkillBehavior {
		activation_type(SkillBehaviorConfig::from_shape(SkillShape::Fn(
			|commands, _, _, _| SkillLayout {
				contact: commands.spawn(()).id(),
				projection: commands.spawn(()).id(),
				on_skill_stop: OnSkillStop::Ignore,
			},
		)))
	}

	fn setup() -> Result<(App, Entity), MissingLastUpdate> {
		let mut app = App::new().single_threaded(Update);
		let agent = app
			.world_mut()
			.spawn(_Executor::new().with_mock(|mock| {
				mock.expect_schedule().return_const(());
				mock.expect_flush().return_const(());
			}))
			.id();

		app.init_resource::<Time<Real>>();
		app.tick_time(Duration::ZERO)?;
		app.update();
		app.add_systems(
			Update,
			advance_active_skill::<_Dequeue, Query<Mut<_Facing>>, _Executor, Real>,
		);

		Ok((app, agent))
	}

	#[test]
	fn call_update_with_delta() -> Result<(), MissingLastUpdate> {
		let (mut app, agent) = setup()?;
		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					Mock_Skill::new_mock(|mock| {
						mock.expect_behavior()
							.return_const((SlotKey(0), RunSkillBehavior::default()));
						mock.expect_updated_states()
							.times(1)
							.with(eq(Duration::from_millis(100)))
							.return_const(HashSet::<StateMeta<SkillState>>::default());
					})
				})),
			},
			Transform::default(),
			_Facing::default(),
		));

		app.tick_time(Duration::from_millis(100))?;
		app.update();
		Ok(())
	}

	#[test]
	fn clear_queue_of_active() -> Result<(), MissingLastUpdate> {
		let (mut app, agent) = setup()?;
		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					Mock_Skill::new_mock(|mock| {
						mock.expect_behavior()
							.return_const((SlotKey(0), RunSkillBehavior::default()));
						mock.expect_updated_states().return_const(
							HashSet::<StateMeta<SkillState>>::from([StateMeta::Done]),
						);
					})
				})),
			},
			Transform::default(),
			_Facing::default(),
		));

		app.update();

		let agent = app.world().entity(agent);

		assert!(agent.get::<_Dequeue>().unwrap().active.is_none());
		Ok(())
	}

	#[test]
	fn do_not_remove_skill_when_not_done() -> Result<(), MissingLastUpdate> {
		let (mut app, agent) = setup()?;
		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					Mock_Skill::new_mock(|mock| {
						mock.expect_behavior()
							.return_const((SlotKey(0), RunSkillBehavior::default()));
						mock.expect_updated_states().return_const(
							HashSet::<StateMeta<SkillState>>::from([StateMeta::In(
								SkillState::Active,
							)]),
						);
					})
				})),
			},
			Transform::default(),
			_Facing::default(),
		));

		app.update();

		let agent = app.world().entity(agent);

		assert!(agent.get::<_Dequeue>().unwrap().active.is_some());
		Ok(())
	}

	#[test]
	fn run_on_active() -> Result<(), MissingLastUpdate> {
		let (mut app, agent) = setup()?;
		app.world_mut().entity_mut(agent).insert((
			_Executor::new().with_mock(|mock| {
				mock.expect_flush().return_const(());
				mock.expect_schedule()
					.times(1)
					.withf(|slot_key, start| {
						assert_eq!(
							(
								&SlotKey::from(PlayerSlot::Upper(Side::Left)),
								&skill_behavior(RunSkillBehavior::OnActive)
							),
							(slot_key, start),
						);
						true
					})
					.return_const(());
			}),
			_Dequeue {
				active: Some(Box::new(|| {
					Mock_Skill::new_mock(|mock| {
						mock.expect_behavior().returning(|| {
							(
								SlotKey::from(PlayerSlot::Upper(Side::Left)),
								skill_behavior(RunSkillBehavior::OnActive),
							)
						});
						mock.expect_updated_states().return_const(
							HashSet::<StateMeta<SkillState>>::from([StateMeta::Entering(
								SkillState::Active,
							)]),
						);
					})
				})),
			},
			_Facing::default(),
		));

		app.update();
		Ok(())
	}

	#[test]
	fn run_on_aim() -> Result<(), MissingLastUpdate> {
		let (mut app, agent) = setup()?;
		app.world_mut().entity_mut(agent).insert((
			_Executor::new().with_mock(|mock| {
				mock.expect_flush().return_const(());
				mock.expect_schedule()
					.times(1)
					.withf(|slot_key, start| {
						assert_eq!(
							(
								&SlotKey::from(PlayerSlot::Lower(Side::Left)),
								&skill_behavior(RunSkillBehavior::OnAim)
							),
							(slot_key, start),
						);
						true
					})
					.return_const(());
			}),
			_Dequeue {
				active: Some(Box::new(|| {
					Mock_Skill::new_mock(|mock| {
						mock.expect_behavior().returning(|| {
							(
								SlotKey::from(PlayerSlot::Lower(Side::Left)),
								skill_behavior(RunSkillBehavior::OnAim),
							)
						});
						mock.expect_updated_states().return_const(
							HashSet::<StateMeta<SkillState>>::from([StateMeta::Entering(
								SkillState::Aim,
							)]),
						);
					})
				})),
			},
			_Facing::default(),
		));

		app.update();
		Ok(())
	}

	#[test]
	fn do_not_run_when_not_activating_this_frame() -> Result<(), MissingLastUpdate> {
		let (mut app, agent) = setup()?;
		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					Mock_Skill::new_mock(|mock| {
						mock.expect_behavior()
							.never()
							.return_const((SlotKey(0), RunSkillBehavior::default()));
						mock.expect_updated_states().return_const(
							HashSet::<StateMeta<SkillState>>::from([StateMeta::In(
								SkillState::Active,
							)]),
						);
					})
				})),
			},
			Transform::default(),
			_Facing::default(),
		));

		app.update();
		Ok(())
	}

	#[test]
	fn flush() -> Result<(), MissingLastUpdate> {
		let (mut app, agent) = setup()?;
		app.world_mut().entity_mut(agent).insert((
			_Executor::new().with_mock(|mock| {
				mock.expect_schedule().return_const(());
				mock.expect_flush().times(1).return_const(());
			}),
			_Dequeue {
				active: Some(Box::new(|| {
					Mock_Skill::new_mock(|mock| {
						mock.expect_behavior()
							.return_const((SlotKey(0), RunSkillBehavior::default()));
						mock.expect_updated_states().return_const(
							HashSet::<StateMeta<SkillState>>::from([StateMeta::Done]),
						);
					})
				})),
			},
			Transform::default(),
			_Facing::default(),
		));

		app.update();
		Ok(())
	}

	#[test]
	fn do_not_stop_when_not_done() -> Result<(), MissingLastUpdate> {
		let (mut app, agent) = setup()?;
		app.world_mut().entity_mut(agent).insert((
			_Executor::new().with_mock(|mock| {
				mock.expect_schedule().return_const(());
				mock.expect_flush().never().return_const(());
			}),
			_Dequeue {
				active: Some(Box::new(|| {
					Mock_Skill::new_mock(|mock| {
						mock.expect_behavior()
							.return_const((SlotKey(0), RunSkillBehavior::default()));
						mock.expect_updated_states().return_const(
							HashSet::<StateMeta<SkillState>>::from([StateMeta::In(
								SkillState::Active,
							)]),
						);
					})
				})),
			},
			Transform::default(),
			_Facing::default(),
		));

		app.update();
		Ok(())
	}

	#[test]
	fn apply_facing() -> Result<(), MissingLastUpdate> {
		let (mut app, agent) = setup()?;
		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					Mock_Skill::new_mock(|mock| {
						mock.expect_behavior()
							.return_const((SlotKey(0), RunSkillBehavior::default()));
						mock.expect_updated_states().return_const(
							HashSet::<StateMeta<SkillState>>::from([StateMeta::Entering(
								SkillState::Aim,
							)]),
						);
					})
				})),
			},
			Transform::default(),
			_Facing::new().with_mock(|mock| {
				mock.expect_override_face()
					.times(1)
					.with(eq(Face::Target))
					.return_const(());
				mock.expect_stop_override_face().never();
			}),
		));

		app.update();
		Ok(())
	}

	#[test]
	fn do_not_apply_facing_when_not_beginning_to_aim() -> Result<(), MissingLastUpdate> {
		let (mut app, agent) = setup()?;
		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					Mock_Skill::new_mock(|mock| {
						mock.expect_behavior()
							.return_const((SlotKey(0), RunSkillBehavior::default()));
						mock.expect_updated_states().return_const(
							HashSet::<StateMeta<SkillState>>::from([StateMeta::In(
								SkillState::Aim,
							)]),
						);
					})
				})),
			},
			Transform::default(),
			_Facing::new().with_mock(|mock| {
				mock.expect_override_face().never();
				mock.expect_stop_override_face().never();
			}),
		));

		app.update();
		Ok(())
	}

	#[test]
	fn apply_facing_override_when_beginning_to_aim() -> Result<(), MissingLastUpdate> {
		let (mut app, agent) = setup()?;
		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					Mock_Skill::new_mock(|mock| {
						mock.expect_behavior()
							.return_const((SlotKey(0), RunSkillBehavior::default()));
						mock.expect_updated_states().return_const(
							HashSet::<StateMeta<SkillState>>::from([StateMeta::Entering(
								SkillState::Aim,
							)]),
						);
					})
				})),
			},
			Transform::from_xyz(-1., -2., -3.),
			_Facing::new().with_mock(|mock| {
				mock.expect_override_face()
					.times(1)
					.with(eq(Face::Target))
					.return_const(());
				mock.expect_stop_override_face().never();
			}),
		));

		app.update();
		Ok(())
	}

	#[test]
	fn stop_facing_override_when_no_skills_active() -> Result<(), MissingLastUpdate> {
		let (mut app, agent) = setup()?;
		app.world_mut().entity_mut(agent).insert((
			_Dequeue { active: None },
			Transform::from_xyz(-1., -2., -3.),
			_Facing::new().with_mock(|mock| {
				mock.expect_override_face().never();
				mock.expect_stop_override_face().times(1).return_const(());
			}),
		));

		app.update();
		Ok(())
	}

	#[test]
	fn do_not_mutable_deref_executer_when_skill_states_empty() -> Result<(), MissingLastUpdate> {
		let (mut app, agent) = setup()?;
		app = app.single_threaded(PostUpdate);
		let entity = app
			.world_mut()
			.entity_mut(agent)
			.insert((
				_Dequeue {
					active: Some(Box::new(move || {
						Mock_Skill::new_mock(|mock| {
							mock.expect_behavior()
								.return_const((SlotKey(0), RunSkillBehavior::default()));
							mock.expect_updated_states()
								.return_const(HashSet::<StateMeta<SkillState>>::from([]));
						})
					})),
				},
				Transform::default(),
			))
			.id();

		app.add_systems(PostUpdate, IsChanged::<_Executor>::detect);
		app.update();
		app.update();

		assert_eq!(
			Some(&IsChanged::<_Executor>::FALSE),
			app.world().entity(entity).get::<IsChanged<_Executor>>()
		);
		Ok(())
	}
}
