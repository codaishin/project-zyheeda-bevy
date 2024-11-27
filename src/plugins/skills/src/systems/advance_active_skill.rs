use crate::{
	skills::{Animate, RunSkillBehavior, SkillState},
	slot_key::SlotKey,
	traits::{Flush, GetActiveSkill, GetAnimation, GetSkillBehavior, Schedule},
};
use behaviors::components::{Face, OverrideFace};
use bevy::{ecs::system::EntityCommands, prelude::*};
use common::traits::{
	animation::{AnimationPriority, StartAnimation, StopAnimation},
	state_duration::{StateMeta, StateUpdate},
};
use std::time::Duration;

#[derive(PartialEq)]
enum Advancement {
	Finished,
	InProcess,
}

#[derive(Component)]
pub struct SideEffectsCleared;

#[derive(Debug, PartialEq)]
struct UseSkill;

impl From<UseSkill> for AnimationPriority {
	fn from(_: UseSkill) -> Self {
		AnimationPriority::High
	}
}

type Components<'a, TGetSkill, TAnimationDispatch, TSkillExecutor> = (
	Entity,
	&'a mut TGetSkill,
	&'a mut TAnimationDispatch,
	&'a mut TSkillExecutor,
	Option<&'a SideEffectsCleared>,
);

pub(crate) fn advance_active_skill<
	TGetSkill: GetActiveSkill<SkillState> + Component,
	TAnimationDispatch: Component + StartAnimation + StopAnimation,
	TSkillExecutor: Component + Schedule<RunSkillBehavior> + Flush,
	TTime: Send + Sync + Default + 'static,
>(
	time: Res<Time<TTime>>,
	mut commands: Commands,
	mut agents: Query<Components<TGetSkill, TAnimationDispatch, TSkillExecutor>>,
) {
	let delta = time.delta();

	for (entity, mut dequeue, animation_dispatch, skill_executer, cleared) in &mut agents {
		let Some(agent) = commands.get_entity(entity) else {
			continue;
		};
		let advancement = match dequeue.get_active() {
			Some(skill) => advance(skill, agent, animation_dispatch, skill_executer, delta),
			None if is_not(cleared) => clear_side_effects(agent, animation_dispatch),
			_ => Advancement::InProcess,
		};

		if advancement == Advancement::InProcess {
			continue;
		}

		dequeue.clear_active();
	}
}

fn is_not(cleared: Option<&SideEffectsCleared>) -> bool {
	cleared.is_none()
}

fn clear_side_effects<TAnimationDispatch: StopAnimation>(
	mut agent: EntityCommands,
	mut animation_dispatch: Mut<TAnimationDispatch>,
) -> Advancement {
	agent.remove::<OverrideFace>();
	agent.try_insert(SideEffectsCleared);
	animation_dispatch.stop_animation(UseSkill);

	Advancement::InProcess
}

fn advance<
	TAnimationDispatch: StartAnimation + StopAnimation,
	TSkillExecutor: Component + Schedule<RunSkillBehavior> + Flush,
>(
	mut skill: (impl GetSkillBehavior + GetAnimation + StateUpdate<SkillState>),
	mut agent: EntityCommands,
	animation_dispatch: Mut<TAnimationDispatch>,
	mut skill_executer: Mut<TSkillExecutor>,
	delta: Duration,
) -> Advancement {
	let skill = &mut skill;
	let agent = &mut agent;
	let states = skill.update_state(delta);

	agent.remove::<SideEffectsCleared>();

	if states.contains(&StateMeta::Entering(SkillState::Aim)) {
		agent.try_insert(OverrideFace(Face::Cursor));
		animate(skill, animation_dispatch);
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

fn animate<TAnimationDispatch: StartAnimation + StopAnimation>(
	skill: &mut (impl GetSkillBehavior + GetAnimation + StateUpdate<SkillState>),
	mut dispatch: Mut<TAnimationDispatch>,
) {
	match skill.animate() {
		Animate::Some(animation) => dispatch.start_animation(UseSkill, animation),
		Animate::None => dispatch.stop_animation(UseSkill),
		Animate::Ignore => {}
	}
}

fn run_on_aim<TSkill: GetSkillBehavior>(skill: &TSkill) -> Option<(SlotKey, RunSkillBehavior)> {
	let (slot_key, behavior) = skill.behavior();
	match &behavior {
		RunSkillBehavior::OnAim(_) => Some((slot_key, behavior)),
		_ => None,
	}
}

fn run_on_active<TSkill: GetSkillBehavior>(skill: &TSkill) -> Option<(SlotKey, RunSkillBehavior)> {
	let (slot_key, behavior) = skill.behavior();
	match &behavior {
		RunSkillBehavior::OnActive(_) => Some((slot_key, behavior)),
		_ => None,
	}
}

fn schedule_start<TSkillExecutor: Schedule<RunSkillBehavior>, TSkill: GetSkillBehavior>(
	executer: &mut Mut<TSkillExecutor>,
	skill: &TSkill,
	get_start_fn: fn(&TSkill) -> Option<(SlotKey, RunSkillBehavior)>,
) {
	let Some((slot_key, start_fn)) = get_start_fn(skill) else {
		return;
	};
	executer.schedule(slot_key, start_fn);
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		behaviors::{
			build_skill_shape::{BuildSkillShape, OnSkillStop},
			SkillBehaviorConfig,
		},
		traits::{skill_builder::SkillShape, GetAnimation, GetSkillBehavior},
	};
	use behaviors::components::{Face, OverrideFace};
	use bevy::{
		prelude::{App, Transform, Update},
		time::{Real, Time},
	};
	use common::{
		components::Side,
		simple_init,
		test_tools::utils::{Changed, SingleThreadedApp, TickTime},
		traits::{
			animation::{Animation, PlayMode},
			load_asset::Path,
			mock::Mock,
			nested_mock::NestedMocks,
		},
	};
	use macros::NestedMocks;
	use mockall::{mock, predicate::eq};
	use std::{collections::HashSet, ops::DerefMut, time::Duration};

	#[derive(Component, Default)]
	struct _Dequeue {
		pub active: Option<Box<dyn FnMut() -> Mock_Skill + Sync + Send>>,
	}

	impl GetActiveSkill<SkillState> for _Dequeue {
		fn clear_active(&mut self) {
			self.active = None;
		}

		fn get_active(
			&mut self,
		) -> Option<impl GetSkillBehavior + GetAnimation + StateUpdate<SkillState>> {
			self.active.as_mut().map(|f| f())
		}
	}

	mock! {
		_Skill {}
		impl StateUpdate<SkillState> for _Skill {
			fn update_state(&mut self, delta: Duration) -> HashSet<StateMeta<SkillState>>;
		}
		impl GetSkillBehavior for _Skill {
			fn behavior<'a>(&self) -> (SlotKey, RunSkillBehavior);
		}
		impl GetAnimation for _Skill {
			fn animate(&self) -> Animate<Animation>;
		}
	}

	simple_init!(Mock_Skill);

	#[derive(Component, NestedMocks)]
	struct _AnimationDispatch {
		mock: Mock_AnimationDispatch,
	}

	impl StartAnimation for _AnimationDispatch {
		fn start_animation<TLayer>(&mut self, layer: TLayer, animation: Animation)
		where
			TLayer: Into<AnimationPriority> + 'static,
		{
			self.mock.start_animation(layer, animation)
		}
	}

	impl StopAnimation for _AnimationDispatch {
		fn stop_animation<TLayer>(&mut self, layer: TLayer)
		where
			TLayer: Into<AnimationPriority> + 'static,
		{
			self.mock.stop_animation(layer)
		}
	}

	mock! {
		_AnimationDispatch {}
		impl StartAnimation for _AnimationDispatch {
			fn start_animation<TLayer>(&mut self, layer: TLayer, animation: Animation)
			where
				TLayer: Into<AnimationPriority> + 'static;
		}
		impl StopAnimation for _AnimationDispatch {
			fn stop_animation<TLayer>(&mut self, layer: TLayer) where
				TLayer: Into<AnimationPriority> + 'static;
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
		activation_type(SkillBehaviorConfig::from_shape(BuildSkillShape::Fn(
			|commands, _, _, _| SkillShape {
				contact: commands.spawn_empty().id(),
				projection: commands.spawn_empty().id(),
				on_skill_stop: OnSkillStop::Ignore,
			},
		)))
	}

	fn setup() -> (App, Entity) {
		let mut app = App::new().single_threaded(Update);
		let agent = app
			.world_mut()
			.spawn((
				_AnimationDispatch::new().with_mock(|mock| {
					mock.expect_start_animation::<UseSkill>().return_const(());
					mock.expect_stop_animation::<UseSkill>().return_const(());
				}),
				_Executor::new().with_mock(|mock| {
					mock.expect_schedule().return_const(());
					mock.expect_flush().return_const(());
				}),
			))
			.id();

		app.init_resource::<Time<Real>>();
		app.tick_time(Duration::ZERO);
		app.update();
		app.add_systems(
			Update,
			advance_active_skill::<_Dequeue, _AnimationDispatch, _Executor, Real>,
		);

		(app, agent)
	}

	#[test]
	fn call_update_with_delta() {
		let (mut app, agent) = setup();
		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					Mock_Skill::new_mock(|mock| {
						mock.expect_animate().return_const(Animate::Ignore);
						mock.expect_behavior()
							.return_const((SlotKey::default(), RunSkillBehavior::default()));
						mock.expect_update_state()
							.times(1)
							.with(eq(Duration::from_millis(100)))
							.return_const(HashSet::<StateMeta<SkillState>>::default());
					})
				})),
			},
			Transform::default(),
		));

		app.tick_time(Duration::from_millis(100));
		app.update();
	}

	#[test]
	fn insert_animation_when_aim_begins() {
		let (mut app, agent) = setup();
		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(move || {
					Mock_Skill::new_mock(|mock| {
						mock.expect_animate()
							.return_const(Animate::Some(Animation::new(
								Path::from("42"),
								PlayMode::Repeat,
							)));
						mock.expect_behavior()
							.return_const((SlotKey::default(), RunSkillBehavior::default()));
						mock.expect_update_state().return_const(
							HashSet::<StateMeta<SkillState>>::from([StateMeta::Entering(
								SkillState::Aim,
							)]),
						);
					})
				})),
			},
			Transform::default(),
			_AnimationDispatch::new().with_mock(|mock| {
				mock.expect_stop_animation::<UseSkill>().return_const(());
				mock.expect_start_animation()
					.times(1)
					.with(
						eq(UseSkill),
						eq(Animation::new(Path::from("42"), PlayMode::Repeat)),
					)
					.return_const(());
			}),
		));

		app.update();
	}

	#[test]
	fn do_not_insert_animation_when_not_beginning_to_aim() {
		let (mut app, agent) = setup();
		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(move || {
					Mock_Skill::new_mock(|mock| {
						mock.expect_animate()
							.return_const(Animate::Some(Animation::new(
								Path::from("42"),
								PlayMode::Repeat,
							)));
						mock.expect_behavior()
							.return_const((SlotKey::default(), RunSkillBehavior::default()));
						mock.expect_update_state().return_const(
							HashSet::<StateMeta<SkillState>>::from([
								StateMeta::In(SkillState::Aim),
								StateMeta::Entering(SkillState::Active),
								StateMeta::In(SkillState::Active),
								StateMeta::Done,
							]),
						);
					})
				})),
			},
			Transform::default(),
			_AnimationDispatch::new().with_mock(|mock| {
				mock.expect_stop_animation::<UseSkill>().return_const(());
				mock.expect_start_animation::<UseSkill>()
					.never()
					.return_const(());
			}),
		));

		app.update();
	}

	#[test]
	fn stop_animation_on_when_beginning_to_aim_and_animate_is_none() {
		let (mut app, agent) = setup();
		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(move || {
					Mock_Skill::new_mock(|mock| {
						mock.expect_animate().return_const(Animate::None);
						mock.expect_behavior()
							.return_const((SlotKey::default(), RunSkillBehavior::default()));
						mock.expect_update_state().return_const(
							HashSet::<StateMeta<SkillState>>::from([StateMeta::Entering(
								SkillState::Aim,
							)]),
						);
					})
				})),
			},
			Transform::default(),
			_AnimationDispatch::new().with_mock(|mock| {
				mock.expect_start_animation::<UseSkill>().return_const(());
				mock.expect_stop_animation::<UseSkill>()
					.times(1)
					.return_const(());
			}),
		));

		app.update();
	}

	#[test]
	fn do_not_stop_animation_when_not_beginning_to_aim_and_animate_is_none() {
		let (mut app, agent) = setup();
		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(move || {
					Mock_Skill::new_mock(|skill| {
						skill.expect_animate().return_const(Animate::None);
						skill
							.expect_behavior()
							.return_const((SlotKey::default(), RunSkillBehavior::default()));
						skill.expect_update_state().return_const(
							HashSet::<StateMeta<SkillState>>::from([
								StateMeta::In(SkillState::Aim),
								StateMeta::Entering(SkillState::Active),
								StateMeta::In(SkillState::Active),
								StateMeta::Done,
							]),
						);
					})
				})),
			},
			Transform::default(),
			_AnimationDispatch::new().with_mock(|mock| {
				mock.expect_start_animation::<UseSkill>().return_const(());
				mock.expect_stop_animation::<UseSkill>()
					.never()
					.return_const(());
			}),
		));

		app.update();
	}

	#[test]
	fn do_not_start_or_stop_animation_when_beginning_to_aim_and_animate_is_ignore() {
		let (mut app, agent) = setup();
		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(move || {
					Mock_Skill::new_mock(|mock| {
						mock.expect_animate().return_const(Animate::Ignore);
						mock.expect_behavior()
							.return_const((SlotKey::default(), RunSkillBehavior::default()));
						mock.expect_update_state().return_const(
							HashSet::<StateMeta<SkillState>>::from([StateMeta::Entering(
								SkillState::Aim,
							)]),
						);
					})
				})),
			},
			Transform::default(),
			_AnimationDispatch::new().with_mock(|mock| {
				mock.expect_start_animation::<UseSkill>()
					.never()
					.return_const(());
				mock.expect_stop_animation::<UseSkill>()
					.never()
					.return_const(());
			}),
		));

		app.update();
	}

	#[test]
	fn remove_animation_when_no_active_skill() {
		let (mut app, agent) = setup();
		app.world_mut().entity_mut(agent).insert((
			_Dequeue { active: None },
			Transform::default(),
			_AnimationDispatch::new().with_mock(|mock| {
				mock.expect_start_animation::<UseSkill>().return_const(());
				mock.expect_stop_animation::<UseSkill>()
					.times(1)
					.return_const(());
			}),
		));

		app.update();
	}

	#[test]
	fn do_not_remove_animation_when_some_active_skill() {
		let (mut app, agent) = setup();
		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					Mock_Skill::new_mock(|mock| {
						mock.expect_animate().return_const(Animate::None);
						mock.expect_behavior()
							.return_const((SlotKey::default(), RunSkillBehavior::default()));
						mock.expect_update_state().return_const(HashSet::default());
					})
				})),
			},
			Transform::default(),
			_AnimationDispatch::new().with_mock(|mock| {
				mock.expect_start_animation::<UseSkill>().return_const(());
				mock.expect_stop_animation::<UseSkill>()
					.never()
					.return_const(());
			}),
		));

		app.update();
	}

	#[test]
	fn remove_animation_only_once_when_no_active_skill() {
		let (mut app, agent) = setup();
		app.world_mut().entity_mut(agent).insert((
			_Dequeue { active: None },
			Transform::default(),
			_AnimationDispatch::new().with_mock(|mock| {
				mock.expect_start_animation::<UseSkill>().return_const(());
				mock.expect_stop_animation::<UseSkill>()
					.times(1)
					.return_const(());
			}),
		));

		app.update();
		app.update();
	}

	#[test]
	fn remove_animation_only_once_even_when_mutably_dereferenced() {
		let (mut app, agent) = setup();
		let entity = app
			.world_mut()
			.entity_mut(agent)
			.insert((
				_Dequeue { active: None },
				Transform::default(),
				_AnimationDispatch::new().with_mock(|mock| {
					mock.expect_start_animation::<UseSkill>().return_const(());
					mock.expect_stop_animation::<UseSkill>()
						.times(1)
						.return_const(());
				}),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<_Dequeue>()
			.unwrap()
			.deref_mut();
		app.update();
	}

	#[test]
	fn remove_animation_again_when_after_another_active_skill_done() {
		let (mut app, agent) = setup();
		let entity = app
			.world_mut()
			.entity_mut(agent)
			.insert((
				_Dequeue { active: None },
				Transform::default(),
				_AnimationDispatch::new().with_mock(|mock| {
					mock.expect_start_animation::<UseSkill>().return_const(());
					mock.expect_stop_animation::<UseSkill>()
						.times(2)
						.return_const(());
				}),
			))
			.id();

		app.update();
		let mut dequeue = app.world_mut().entity_mut(entity);
		let mut dequeue = dequeue.get_mut::<_Dequeue>().unwrap();
		dequeue.active = Some(Box::new(|| {
			Mock_Skill::new_mock(|mock| {
				mock.expect_animate().return_const(Animate::None);
				mock.expect_behavior()
					.return_const((SlotKey::default(), RunSkillBehavior::default()));
				mock.expect_update_state()
					.return_const(HashSet::<StateMeta<SkillState>>::from([]));
			})
		}));
		app.update();
		let mut dequeue = app.world_mut().entity_mut(entity);
		let mut dequeue = dequeue.get_mut::<_Dequeue>().unwrap();
		dequeue.active = None;
		app.update();
	}

	#[test]
	fn clear_queue_of_active() {
		let (mut app, agent) = setup();
		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					Mock_Skill::new_mock(|mock| {
						mock.expect_animate().return_const(Animate::None);
						mock.expect_behavior()
							.return_const((SlotKey::default(), RunSkillBehavior::default()));
						mock.expect_update_state().return_const(
							HashSet::<StateMeta<SkillState>>::from([StateMeta::Done]),
						);
					})
				})),
			},
			Transform::default(),
		));

		app.update();

		let agent = app.world().entity(agent);

		assert!(agent.get::<_Dequeue>().unwrap().active.is_none());
	}

	#[test]
	fn do_not_remove_skill_when_not_done() {
		let (mut app, agent) = setup();
		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					Mock_Skill::new_mock(|mock| {
						mock.expect_animate().return_const(Animate::None);
						mock.expect_behavior()
							.return_const((SlotKey::default(), RunSkillBehavior::default()));
						mock.expect_update_state().return_const(
							HashSet::<StateMeta<SkillState>>::from([StateMeta::In(
								SkillState::Active,
							)]),
						);
					})
				})),
			},
			Transform::default(),
		));

		app.update();

		let agent = app.world().entity(agent);

		assert!(agent.get::<_Dequeue>().unwrap().active.is_some());
	}

	#[test]
	fn run_on_active() {
		let (mut app, agent) = setup();
		app.world_mut().entity_mut(agent).insert((
			_Executor::new().with_mock(|mock| {
				mock.expect_flush().return_const(());
				mock.expect_schedule()
					.times(1)
					.withf(|slot_key, start| {
						assert_eq!(
							(
								&SlotKey::TopHand(Side::Left),
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
						mock.expect_animate().return_const(Animate::None);
						mock.expect_behavior().returning(|| {
							(
								SlotKey::TopHand(Side::Left),
								skill_behavior(RunSkillBehavior::OnActive),
							)
						});
						mock.expect_update_state().return_const(
							HashSet::<StateMeta<SkillState>>::from([StateMeta::Entering(
								SkillState::Active,
							)]),
						);
					})
				})),
			},
		));

		app.update();
	}

	#[test]
	fn run_on_aim() {
		let (mut app, agent) = setup();
		app.world_mut().entity_mut(agent).insert((
			_Executor::new().with_mock(|mock| {
				mock.expect_flush().return_const(());
				mock.expect_schedule()
					.times(1)
					.withf(|slot_key, start| {
						assert_eq!(
							(
								&SlotKey::BottomHand(Side::Left),
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
						mock.expect_animate().return_const(Animate::None);
						mock.expect_behavior().returning(|| {
							(
								SlotKey::BottomHand(Side::Left),
								skill_behavior(RunSkillBehavior::OnAim),
							)
						});
						mock.expect_update_state().return_const(
							HashSet::<StateMeta<SkillState>>::from([StateMeta::Entering(
								SkillState::Aim,
							)]),
						);
					})
				})),
			},
		));

		app.update();
	}

	#[test]
	fn do_not_run_when_not_activating_this_frame() {
		let (mut app, agent) = setup();
		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					Mock_Skill::new_mock(|mock| {
						mock.expect_animate().return_const(Animate::None);
						mock.expect_behavior()
							.never()
							.return_const((SlotKey::default(), RunSkillBehavior::default()));
						mock.expect_update_state().return_const(
							HashSet::<StateMeta<SkillState>>::from([StateMeta::In(
								SkillState::Active,
							)]),
						);
					})
				})),
			},
			Transform::default(),
		));

		app.update();
	}

	#[test]
	fn flush() {
		let (mut app, agent) = setup();
		app.world_mut().entity_mut(agent).insert((
			_Executor::new().with_mock(|mock| {
				mock.expect_schedule().return_const(());
				mock.expect_flush().times(1).return_const(());
			}),
			_Dequeue {
				active: Some(Box::new(|| {
					Mock_Skill::new_mock(|mock| {
						mock.expect_animate().return_const(Animate::None);
						mock.expect_behavior()
							.return_const((SlotKey::default(), RunSkillBehavior::default()));
						mock.expect_update_state().return_const(
							HashSet::<StateMeta<SkillState>>::from([StateMeta::Done]),
						);
					})
				})),
			},
			Transform::default(),
		));

		app.update();
	}

	#[test]
	fn do_not_stop_when_not_done() {
		let (mut app, agent) = setup();
		app.world_mut().entity_mut(agent).insert((
			_Executor::new().with_mock(|mock| {
				mock.expect_schedule().return_const(());
				mock.expect_flush().never().return_const(());
			}),
			_Dequeue {
				active: Some(Box::new(|| {
					Mock_Skill::new_mock(|mock| {
						mock.expect_animate().return_const(Animate::None);
						mock.expect_behavior()
							.return_const((SlotKey::default(), RunSkillBehavior::default()));
						mock.expect_update_state().return_const(
							HashSet::<StateMeta<SkillState>>::from([StateMeta::In(
								SkillState::Active,
							)]),
						);
					})
				})),
			},
			Transform::default(),
		));

		app.update();
	}

	#[test]
	fn apply_facing() {
		let (mut app, agent) = setup();
		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					Mock_Skill::new_mock(|mock| {
						mock.expect_animate().return_const(Animate::None);
						mock.expect_behavior()
							.return_const((SlotKey::default(), RunSkillBehavior::default()));
						mock.expect_update_state().return_const(
							HashSet::<StateMeta<SkillState>>::from([StateMeta::Entering(
								SkillState::Aim,
							)]),
						);
					})
				})),
			},
			Transform::default(),
		));

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(
			Some(&OverrideFace(Face::Cursor)),
			agent.get::<OverrideFace>()
		);
	}

	#[test]
	fn do_not_apply_facing_when_not_beginning_to_aim() {
		let (mut app, agent) = setup();
		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					Mock_Skill::new_mock(|mock| {
						mock.expect_animate().return_const(Animate::None);
						mock.expect_behavior()
							.return_const((SlotKey::default(), RunSkillBehavior::default()));
						mock.expect_update_state().return_const(
							HashSet::<StateMeta<SkillState>>::from([StateMeta::In(
								SkillState::Aim,
							)]),
						);
					})
				})),
			},
			Transform::default(),
		));

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(None, agent.get::<OverrideFace>());
	}

	#[test]
	fn apply_facing_override_when_beginning_to_aim() {
		let (mut app, agent) = setup();
		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					Mock_Skill::new_mock(|mock| {
						mock.expect_animate().return_const(Animate::None);
						mock.expect_behavior()
							.return_const((SlotKey::default(), RunSkillBehavior::default()));
						mock.expect_update_state().return_const(
							HashSet::<StateMeta<SkillState>>::from([StateMeta::Entering(
								SkillState::Aim,
							)]),
						);
					})
				})),
			},
			Transform::from_xyz(-1., -2., -3.),
		));

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(
			Some(&OverrideFace(Face::Cursor)),
			agent.get::<OverrideFace>()
		);
	}

	#[test]
	fn no_facing_override_when_no_skill() {
		let (mut app, agent) = setup();
		app.world_mut().entity_mut(agent).insert((
			_Dequeue { active: None },
			Transform::from_xyz(-1., -2., -3.),
			OverrideFace(Face::Cursor),
		));

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(None, agent.get::<OverrideFace>());
	}

	#[test]
	fn do_not_mutable_deref_animation_dispatch_when_no_animation_used() {
		let (mut app, agent) = setup();
		app = app.single_threaded(PostUpdate);
		let entity = app
			.world_mut()
			.entity_mut(agent)
			.insert((
				_Dequeue {
					active: Some(Box::new(move || {
						Mock_Skill::new_mock(|mock| {
							mock.expect_animate().return_const(Animate::Ignore);
							mock.expect_behavior()
								.return_const((SlotKey::default(), RunSkillBehavior::default()));
							mock.expect_update_state().return_const(
								HashSet::<StateMeta<SkillState>>::from([StateMeta::Entering(
									SkillState::Aim,
								)]),
							);
						})
					})),
				},
				Transform::default(),
				Changed::<_AnimationDispatch>::new(false),
			))
			.id();

		app.add_systems(PostUpdate, Changed::<_AnimationDispatch>::detect);
		app.update();
		app.update();

		assert_eq!(
			Some(&false),
			app.world()
				.entity(entity)
				.get::<Changed<_AnimationDispatch>>()
				.map(|Changed { changed, .. }| changed)
		)
	}

	#[test]
	fn do_not_mutable_deref_executer_when_skill_states_empty() {
		let (mut app, agent) = setup();
		app = app.single_threaded(PostUpdate);
		let entity = app
			.world_mut()
			.entity_mut(agent)
			.insert((
				_Dequeue {
					active: Some(Box::new(move || {
						Mock_Skill::new_mock(|mock| {
							mock.expect_animate().return_const(Animate::Ignore);
							mock.expect_behavior()
								.return_const((SlotKey::default(), RunSkillBehavior::default()));
							mock.expect_update_state()
								.return_const(HashSet::<StateMeta<SkillState>>::from([]));
						})
					})),
				},
				Transform::default(),
				Changed::<_Executor>::new(false),
			))
			.id();

		app.add_systems(PostUpdate, Changed::<_Executor>::detect);
		app.update();
		app.update();

		assert_eq!(
			Some(&false),
			app.world()
				.entity(entity)
				.get::<Changed<_Executor>>()
				.map(|Changed { changed, .. }| changed)
		)
	}
}
