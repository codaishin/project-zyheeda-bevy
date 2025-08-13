use crate::{
	skills::{AnimationStrategy, RunSkillBehavior, SkillState},
	traits::{Flush, GetActiveSkill, GetAnimationStrategy, GetSkillBehavior, Schedule},
};
use bevy::{
	ecs::{component::Mutable, system::EntityCommands},
	prelude::*,
};
use common::{
	tools::action_key::slot::SlotKey,
	traits::{
		handles_orientation::{Face, HandlesOrientation},
		handles_player::ConfiguresPlayerSkillAnimations,
		state_duration::{StateMeta, StateUpdate},
	},
};
use std::time::Duration;

#[derive(PartialEq)]
enum Advancement {
	Finished,
	InProcess,
}

#[derive(Component)]
pub struct SkillSideEffectsCleared;

type Components<'a, TGetSkill, TSkillExecutor> = (
	Entity,
	&'a mut TGetSkill,
	&'a mut TSkillExecutor,
	Option<&'a SkillSideEffectsCleared>,
);

pub(crate) fn advance_active_skill<
	TGetSkill: GetActiveSkill<SkillState> + Component<Mutability = Mutable>,
	TPlayerAnimations: ConfiguresPlayerSkillAnimations,
	TOrientation: HandlesOrientation,
	TSkillExecutor: Component<Mutability = Mutable> + Schedule<RunSkillBehavior> + Flush,
	TTime: Send + Sync + Default + 'static,
>(
	time: Res<Time<TTime>>,
	mut commands: Commands,
	mut agents: Query<Components<TGetSkill, TSkillExecutor>>,
) -> Result<(), Vec<TPlayerAnimations::TError>> {
	let delta = time.delta();
	let mut errors = vec![];

	for (entity, mut dequeue, skill_executer, cleared) in &mut agents {
		let Ok(agent) = commands.get_entity(entity) else {
			continue;
		};
		let advancement = match dequeue.get_active() {
			Some(skill) => advance::<TPlayerAnimations, TOrientation, TSkillExecutor>(
				skill,
				agent,
				skill_executer,
				delta,
				&mut errors,
			),
			None if is_not(cleared) => clear_side_effects::<TPlayerAnimations, TOrientation>(agent),
			_ => Advancement::InProcess,
		};

		if advancement == Advancement::InProcess {
			continue;
		}

		dequeue.clear_active();
	}

	if !errors.is_empty() {
		return Err(errors);
	}

	Ok(())
}

fn is_not(cleared: Option<&SkillSideEffectsCleared>) -> bool {
	cleared.is_none()
}

fn clear_side_effects<TPlayerAnimations, TOrientation>(mut agent: EntityCommands) -> Advancement
where
	TPlayerAnimations: ConfiguresPlayerSkillAnimations,
	TOrientation: HandlesOrientation,
{
	agent.remove::<TOrientation::TFaceTemporarily>();
	agent.try_insert((
		SkillSideEffectsCleared,
		TPlayerAnimations::stop_skill_animation(),
	));

	Advancement::InProcess
}

fn advance<TPlayerAnimations, TOrientation, TSkillExecutor>(
	mut skill: (impl GetSkillBehavior + GetAnimationStrategy + StateUpdate<SkillState>),
	mut agent: EntityCommands,
	mut skill_executer: Mut<TSkillExecutor>,
	delta: Duration,
	errors: &mut Vec<TPlayerAnimations::TError>,
) -> Advancement
where
	TPlayerAnimations: ConfiguresPlayerSkillAnimations,
	TOrientation: HandlesOrientation,
	TSkillExecutor: Component + Schedule<RunSkillBehavior> + Flush,
{
	let skill = &mut skill;
	let agent = &mut agent;
	let states = skill.update_state(delta);

	agent.remove::<SkillSideEffectsCleared>();

	if states.contains(&StateMeta::Entering(SkillState::Aim)) {
		agent.try_insert(TOrientation::temporarily(Face::Cursor));
		if let Err(error) = animate::<TPlayerAnimations>(skill, agent) {
			errors.push(error);
		}
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

fn animate<TPlayerAnimations>(
	skill: &(impl GetAnimationStrategy + GetSkillBehavior),
	entity: &mut EntityCommands,
) -> Result<(), TPlayerAnimations::TError>
where
	TPlayerAnimations: ConfiguresPlayerSkillAnimations,
{
	let (slot, ..) = skill.behavior();

	match skill.animation_strategy() {
		AnimationStrategy::Animate => {
			entity.try_insert(TPlayerAnimations::start_skill_animation(slot)?);
		}
		AnimationStrategy::DoNotAnimate => {
			entity.try_insert(TPlayerAnimations::stop_skill_animation());
		}
		AnimationStrategy::None => {}
	}
	Ok(())
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
	use super::*;
	use crate::{
		behaviors::{
			SkillBehaviorConfig,
			build_skill_shape::{BuildSkillShape, OnSkillStop},
		},
		traits::skill_builder::SkillShape,
	};
	use common::{
		errors::Error,
		tools::action_key::slot::{PlayerSlot, Side},
	};
	use macros::NestedMocks;
	use mockall::{mock, predicate::eq};
	use std::{collections::HashSet, ops::DerefMut};
	use testing::{IsChanged, Mock, NestedMocks, SingleThreadedApp, TickTime, simple_init};

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
		) -> Option<impl GetSkillBehavior + GetAnimationStrategy + StateUpdate<SkillState>> {
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
		impl GetAnimationStrategy for _Skill {
			fn animation_strategy(&self) -> AnimationStrategy;
		}
	}

	simple_init!(Mock_Skill);

	struct _Player;

	impl ConfiguresPlayerSkillAnimations for _Player {
		type TAnimationMarker = _SkillAnimation;
		type TError = _AnimationError;

		fn start_skill_animation(
			slot_key: SlotKey,
		) -> Result<Self::TAnimationMarker, Self::TError> {
			Ok(_SkillAnimation::Start(slot_key))
		}

		fn stop_skill_animation() -> Self::TAnimationMarker {
			_SkillAnimation::Stop
		}
	}

	struct _FaultyPlayer;

	impl ConfiguresPlayerSkillAnimations for _FaultyPlayer {
		type TAnimationMarker = _SkillAnimation;
		type TError = _AnimationError;

		fn start_skill_animation(_: SlotKey) -> Result<Self::TAnimationMarker, Self::TError> {
			Err(_AnimationError)
		}

		fn stop_skill_animation() -> Self::TAnimationMarker {
			_SkillAnimation::Stop
		}
	}

	#[derive(Debug, PartialEq)]
	struct _AnimationError;

	impl From<_AnimationError> for Error {
		fn from(_: _AnimationError) -> Self {
			panic!("not used")
		}
	}

	#[derive(Component, Debug, PartialEq)]
	enum _SkillAnimation {
		Start(SlotKey),
		Stop,
	}

	struct _HandlesOrientation;

	impl HandlesOrientation for _HandlesOrientation {
		type TFaceTemporarily = _TempFace;

		fn temporarily(face: Face) -> Self::TFaceTemporarily {
			_TempFace(face)
		}
	}

	#[derive(Component, Debug, PartialEq, Clone)]
	struct _TempFace(Face);

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
				contact: commands.spawn(()).id(),
				projection: commands.spawn(()).id(),
				on_skill_stop: OnSkillStop::Ignore,
			},
		)))
	}

	#[derive(Resource, Debug, PartialEq)]
	struct _Result(Result<(), Vec<_AnimationError>>);

	fn setup<TPlayer>() -> (App, Entity)
	where
		TPlayer: ConfiguresPlayerSkillAnimations<TError = _AnimationError> + 'static,
	{
		let mut app = App::new().single_threaded(Update);
		let agent = app
			.world_mut()
			.spawn(_Executor::new().with_mock(|mock| {
				mock.expect_schedule().return_const(());
				mock.expect_flush().return_const(());
			}))
			.id();

		app.init_resource::<Time<Real>>();
		app.tick_time(Duration::ZERO);
		app.update();
		app.add_systems(
			Update,
			advance_active_skill::<_Dequeue, TPlayer, _HandlesOrientation, _Executor, Real>.pipe(
				|In(r), mut commands: Commands| {
					commands.insert_resource(_Result(r));
				},
			),
		);

		(app, agent)
	}

	#[test]
	fn call_update_with_delta() {
		let (mut app, agent) = setup::<_Player>();
		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					Mock_Skill::new_mock(|mock| {
						mock.expect_animation_strategy()
							.return_const(AnimationStrategy::None);
						mock.expect_behavior()
							.return_const((SlotKey(0), RunSkillBehavior::default()));
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
		let (mut app, agent) = setup::<_Player>();
		let entity = app
			.world_mut()
			.entity_mut(agent)
			.insert((
				_Dequeue {
					active: Some(Box::new(move || {
						Mock_Skill::new_mock(|mock| {
							mock.expect_animation_strategy()
								.return_const(AnimationStrategy::Animate);
							mock.expect_behavior()
								.return_const((SlotKey(42), RunSkillBehavior::default()));
							mock.expect_update_state().return_const(
								HashSet::<StateMeta<SkillState>>::from([StateMeta::Entering(
									SkillState::Aim,
								)]),
							);
						})
					})),
				},
				Transform::default(),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_SkillAnimation::Start(SlotKey(42))),
			app.world().entity(entity).get::<_SkillAnimation>()
		);
	}

	#[test]
	fn return_animation_error() {
		let (mut app, agent) = setup::<_FaultyPlayer>();
		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(move || {
					Mock_Skill::new_mock(|mock| {
						mock.expect_animation_strategy()
							.return_const(AnimationStrategy::Animate);
						mock.expect_behavior()
							.return_const((SlotKey(42), RunSkillBehavior::default()));
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

		assert_eq!(
			Some(&_Result(Err(vec![_AnimationError]))),
			app.world().get_resource::<_Result>()
		);
	}

	#[test]
	fn do_not_insert_animation_when_not_beginning_to_aim() {
		let (mut app, agent) = setup::<_Player>();
		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(move || {
					Mock_Skill::new_mock(|mock| {
						mock.expect_animation_strategy()
							.return_const(AnimationStrategy::Animate);
						mock.expect_behavior()
							.return_const((SlotKey(0), RunSkillBehavior::default()));
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
		));

		app.update();
	}

	#[test]
	fn stop_animation_on_when_beginning_to_aim_and_animate_is_do_not_animate() {
		let (mut app, agent) = setup::<_Player>();
		let entity = app
			.world_mut()
			.entity_mut(agent)
			.insert((
				_Dequeue {
					active: Some(Box::new(move || {
						Mock_Skill::new_mock(|mock| {
							mock.expect_animation_strategy()
								.return_const(AnimationStrategy::DoNotAnimate);
							mock.expect_behavior()
								.return_const((SlotKey(0), RunSkillBehavior::default()));
							mock.expect_update_state().return_const(
								HashSet::<StateMeta<SkillState>>::from([StateMeta::Entering(
									SkillState::Aim,
								)]),
							);
						})
					})),
				},
				Transform::default(),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_SkillAnimation::Stop),
			app.world().entity(entity).get::<_SkillAnimation>(),
		);
	}

	#[test]
	fn do_not_stop_animation_when_not_beginning_to_aim_and_animate_is_do_not_animate() {
		let (mut app, agent) = setup::<_Player>();
		let entity = app
			.world_mut()
			.entity_mut(agent)
			.insert((
				_Dequeue {
					active: Some(Box::new(move || {
						Mock_Skill::new_mock(|skill| {
							skill
								.expect_animation_strategy()
								.return_const(AnimationStrategy::DoNotAnimate);
							skill
								.expect_behavior()
								.return_const((SlotKey(0), RunSkillBehavior::default()));
							skill.expect_update_state().return_const(HashSet::<
								StateMeta<SkillState>,
							>::from([
								StateMeta::In(SkillState::Aim),
								StateMeta::Entering(SkillState::Active),
								StateMeta::In(SkillState::Active),
								StateMeta::Done,
							]));
						})
					})),
				},
				Transform::default(),
			))
			.id();

		app.update();

		assert_eq!(None, app.world().entity(entity).get::<_SkillAnimation>());
	}

	#[test]
	fn do_not_start_or_stop_animation_when_beginning_to_aim_and_animate_is_none() {
		let (mut app, agent) = setup::<_Player>();
		let entity = app
			.world_mut()
			.entity_mut(agent)
			.insert((
				_Dequeue {
					active: Some(Box::new(move || {
						Mock_Skill::new_mock(|mock| {
							mock.expect_animation_strategy()
								.return_const(AnimationStrategy::None);
							mock.expect_behavior()
								.return_const((SlotKey(0), RunSkillBehavior::default()));
							mock.expect_update_state().return_const(
								HashSet::<StateMeta<SkillState>>::from([StateMeta::Entering(
									SkillState::Aim,
								)]),
							);
						})
					})),
				},
				Transform::default(),
			))
			.id();

		app.update();

		assert_eq!(None, app.world().entity(entity).get::<_SkillAnimation>());
	}

	#[test]
	fn stop_animation_when_no_active_skill() {
		let (mut app, agent) = setup::<_Player>();
		let entity = app
			.world_mut()
			.entity_mut(agent)
			.insert((_Dequeue { active: None }, Transform::default()))
			.id();

		app.update();

		assert_eq!(
			Some(&_SkillAnimation::Stop),
			app.world().entity(entity).get::<_SkillAnimation>(),
		);
	}

	#[test]
	fn do_not_stop_animation_when_some_active_skill() {
		let (mut app, agent) = setup::<_Player>();
		let entity = app
			.world_mut()
			.entity_mut(agent)
			.insert((
				_Dequeue {
					active: Some(Box::new(|| {
						Mock_Skill::new_mock(|mock| {
							mock.expect_animation_strategy()
								.return_const(AnimationStrategy::None);
							mock.expect_behavior()
								.return_const((SlotKey(0), RunSkillBehavior::default()));
							mock.expect_update_state().return_const(HashSet::default());
						})
					})),
				},
				Transform::default(),
			))
			.id();

		app.update();

		assert_eq!(None, app.world().entity(entity).get::<_SkillAnimation>(),);
	}

	#[test]
	fn stop_animation_only_once_when_no_active_skill() {
		let (mut app, agent) = setup::<_Player>();
		let entity = app
			.world_mut()
			.entity_mut(agent)
			.insert((_Dequeue { active: None }, Transform::default()))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.remove::<_SkillAnimation>();
		app.update();

		assert_eq!(None, app.world().entity(entity).get::<_SkillAnimation>());
	}

	#[test]
	fn stop_animation_only_once_even_when_dequeue_mutably_dereferenced() {
		let (mut app, agent) = setup::<_Player>();
		let entity = app
			.world_mut()
			.entity_mut(agent)
			.insert((_Dequeue { active: None }, Transform::default()))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<_Dequeue>()
			.unwrap()
			.deref_mut();
		app.world_mut()
			.entity_mut(entity)
			.remove::<_SkillAnimation>();
		app.update();

		assert_eq!(None, app.world().entity(entity).get::<_SkillAnimation>());
	}

	#[test]
	fn stop_animation_again_after_another_active_skill_done() {
		let (mut app, agent) = setup::<_Player>();
		let entity = app
			.world_mut()
			.entity_mut(agent)
			.insert((_Dequeue { active: None }, Transform::default()))
			.id();

		app.update();
		let mut dequeue = app.world_mut().entity_mut(entity);
		let mut dequeue = dequeue.get_mut::<_Dequeue>().unwrap();
		dequeue.active = Some(Box::new(|| {
			Mock_Skill::new_mock(|mock| {
				mock.expect_animation_strategy()
					.return_const(AnimationStrategy::None);
				mock.expect_behavior()
					.return_const((SlotKey(0), RunSkillBehavior::default()));
				mock.expect_update_state()
					.return_const(HashSet::<StateMeta<SkillState>>::from([]));
			})
		}));
		app.update();
		let mut dequeue = app.world_mut().entity_mut(entity);
		let mut dequeue = dequeue.get_mut::<_Dequeue>().unwrap();
		dequeue.active = None;
		app.world_mut()
			.entity_mut(entity)
			.remove::<_SkillAnimation>();
		app.update();

		assert_eq!(
			Some(&_SkillAnimation::Stop),
			app.world().entity(entity).get::<_SkillAnimation>(),
		);
	}

	#[test]
	fn clear_queue_of_active() {
		let (mut app, agent) = setup::<_FaultyPlayer>();
		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					Mock_Skill::new_mock(|mock| {
						mock.expect_animation_strategy()
							.return_const(AnimationStrategy::None);
						mock.expect_behavior()
							.return_const((SlotKey(0), RunSkillBehavior::default()));
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
		let (mut app, agent) = setup::<_FaultyPlayer>();
		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					Mock_Skill::new_mock(|mock| {
						mock.expect_animation_strategy()
							.return_const(AnimationStrategy::None);
						mock.expect_behavior()
							.return_const((SlotKey(0), RunSkillBehavior::default()));
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
		let (mut app, agent) = setup::<_FaultyPlayer>();
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
						mock.expect_animation_strategy()
							.return_const(AnimationStrategy::None);
						mock.expect_behavior().returning(|| {
							(
								SlotKey::from(PlayerSlot::Upper(Side::Left)),
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
		let (mut app, agent) = setup::<_FaultyPlayer>();
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
						mock.expect_animation_strategy()
							.return_const(AnimationStrategy::None);
						mock.expect_behavior().returning(|| {
							(
								SlotKey::from(PlayerSlot::Lower(Side::Left)),
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
		let (mut app, agent) = setup::<_FaultyPlayer>();
		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					Mock_Skill::new_mock(|mock| {
						mock.expect_animation_strategy()
							.return_const(AnimationStrategy::None);
						mock.expect_behavior()
							.never()
							.return_const((SlotKey(0), RunSkillBehavior::default()));
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
		let (mut app, agent) = setup::<_FaultyPlayer>();
		app.world_mut().entity_mut(agent).insert((
			_Executor::new().with_mock(|mock| {
				mock.expect_schedule().return_const(());
				mock.expect_flush().times(1).return_const(());
			}),
			_Dequeue {
				active: Some(Box::new(|| {
					Mock_Skill::new_mock(|mock| {
						mock.expect_animation_strategy()
							.return_const(AnimationStrategy::None);
						mock.expect_behavior()
							.return_const((SlotKey(0), RunSkillBehavior::default()));
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
		let (mut app, agent) = setup::<_FaultyPlayer>();
		app.world_mut().entity_mut(agent).insert((
			_Executor::new().with_mock(|mock| {
				mock.expect_schedule().return_const(());
				mock.expect_flush().never().return_const(());
			}),
			_Dequeue {
				active: Some(Box::new(|| {
					Mock_Skill::new_mock(|mock| {
						mock.expect_animation_strategy()
							.return_const(AnimationStrategy::None);
						mock.expect_behavior()
							.return_const((SlotKey(0), RunSkillBehavior::default()));
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
		let (mut app, agent) = setup::<_FaultyPlayer>();
		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					Mock_Skill::new_mock(|mock| {
						mock.expect_animation_strategy()
							.return_const(AnimationStrategy::None);
						mock.expect_behavior()
							.return_const((SlotKey(0), RunSkillBehavior::default()));
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

		assert_eq!(Some(&_TempFace(Face::Cursor)), agent.get::<_TempFace>());
	}

	#[test]
	fn do_not_apply_facing_when_not_beginning_to_aim() {
		let (mut app, agent) = setup::<_FaultyPlayer>();
		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					Mock_Skill::new_mock(|mock| {
						mock.expect_animation_strategy()
							.return_const(AnimationStrategy::None);
						mock.expect_behavior()
							.return_const((SlotKey(0), RunSkillBehavior::default()));
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
		assert_eq!(None, agent.get::<_TempFace>());
	}

	#[test]
	fn apply_facing_override_when_beginning_to_aim() {
		let (mut app, agent) = setup::<_FaultyPlayer>();
		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					Mock_Skill::new_mock(|mock| {
						mock.expect_animation_strategy()
							.return_const(AnimationStrategy::None);
						mock.expect_behavior()
							.return_const((SlotKey(0), RunSkillBehavior::default()));
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
		assert_eq!(Some(&_TempFace(Face::Cursor)), agent.get::<_TempFace>());
	}

	#[test]
	fn no_facing_override_when_no_skill() {
		let (mut app, agent) = setup::<_FaultyPlayer>();
		app.world_mut().entity_mut(agent).insert((
			_Dequeue { active: None },
			Transform::from_xyz(-1., -2., -3.),
			_TempFace(Face::Cursor),
		));

		app.update();

		let agent = app.world().entity(agent);
		assert_eq!(None, agent.get::<_TempFace>());
	}

	#[test]
	fn do_not_mutable_deref_executer_when_skill_states_empty() {
		let (mut app, agent) = setup::<_FaultyPlayer>();
		app = app.single_threaded(PostUpdate);
		let entity = app
			.world_mut()
			.entity_mut(agent)
			.insert((
				_Dequeue {
					active: Some(Box::new(move || {
						Mock_Skill::new_mock(|mock| {
							mock.expect_animation_strategy()
								.return_const(AnimationStrategy::None);
							mock.expect_behavior()
								.return_const((SlotKey(0), RunSkillBehavior::default()));
							mock.expect_update_state()
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
	}
}
