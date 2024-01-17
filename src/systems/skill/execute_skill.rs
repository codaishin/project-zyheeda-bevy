use crate::{
	behaviors::meta::Spawner,
	components::{Animate, SlotKey, Slots, WaitNext},
	traits::{
		behavior_execution::BehaviorExecution,
		cast_update::{AgeType, CastUpdate, State},
		get_animation::GetAnimation,
	},
};
use bevy::{
	ecs::{component::Component, system::EntityCommands},
	prelude::{Commands, Entity, GlobalTransform, Mut, Query, Res, Time, Transform},
};
use std::time::Duration;

type Skills<'a, TAnimationKey, TSkill> = (
	Entity,
	&'a mut Transform,
	&'a mut TSkill,
	&'a Slots,
	Option<&'a WaitNext>,
	Option<&'a Animate<TAnimationKey>>,
);

pub fn execute_skill<
	TAnimationKey: Copy + Clone + PartialEq + Send + Sync + 'static,
	TSkill: CastUpdate + BehaviorExecution + GetAnimation<TAnimationKey> + Component,
	TTime: Send + Sync + Default + 'static,
>(
	time: Res<Time<TTime>>,
	mut commands: Commands,
	mut agents: Query<Skills<TAnimationKey, TSkill>>,
	transforms: Query<&GlobalTransform>,
) {
	let delta = time.delta();
	for (entity, mut transform, mut skill, slots, wait_next, animate) in &mut agents {
		let agent = &mut commands.entity(entity);
		let transform = &mut transform;
		let skill = &mut skill;
		let transforms = &transforms;

		let state = get_state(skill, &delta, wait_next);

		if state == State::New {
			return handle_new(agent, transform, skill, slots, transforms);
		}

		if state == State::Done {
			return handle_done(agent, skill, animate);
		}

		let State::Activate(age) = state else {
			return;
		};

		handle_active(agent, transform, skill, slots, transforms);

		if age == AgeType::New {
			handle_new(agent, transform, skill, slots, transforms);
		}
	}
}

fn get_state<TSkill: CastUpdate>(
	skill: &mut Mut<TSkill>,
	delta: &Duration,
	wait_next: Option<&WaitNext>,
) -> State {
	if wait_next.is_some() {
		return State::Done;
	}

	skill.update(*delta)
}

fn handle_new<
	TAnimationKey: Clone + Copy + Send + Sync + 'static,
	TSkill: BehaviorExecution + GetAnimation<TAnimationKey>,
>(
	agent: &mut EntityCommands,
	transform: &mut Mut<Transform>,
	skill: &mut Mut<TSkill>,
	slots: &Slots,
	transforms: &Query<&GlobalTransform>,
) {
	if let Some(spawner) = get_spawner(slots, transforms) {
		skill.apply_transform(transform, &spawner);
	};
	agent.insert(skill.animate());
}

fn handle_active<TSkill: BehaviorExecution>(
	agent: &mut EntityCommands,
	agent_transform: &Transform,
	skill: &mut Mut<TSkill>,
	slots: &Slots,
	transforms: &Query<&GlobalTransform>,
) {
	let Some(spawner) = get_spawner(slots, transforms) else {
		return;
	};
	skill.run(agent, agent_transform, &spawner);
}

fn handle_done<
	TAnimationKey: Copy + Clone + Send + Sync + PartialEq + 'static,
	TSkill: CastUpdate + BehaviorExecution + GetAnimation<TAnimationKey> + Component,
>(
	agent: &mut EntityCommands,
	skill: &mut Mut<TSkill>,
	animate: Option<&Animate<TAnimationKey>>,
) {
	agent.remove::<TSkill>();
	if current_animation_is_from_skill(skill, animate) {
		agent.remove::<Animate<TAnimationKey>>();
	}
	agent.insert(WaitNext);
	skill.stop(agent);
}

fn current_animation_is_from_skill<
	TAnimationKey: Clone + Copy + Send + Sync + PartialEq + 'static,
	TSkill: CastUpdate + BehaviorExecution + GetAnimation<TAnimationKey> + Component,
>(
	skill: &mut Mut<TSkill>,
	animate: Option<&Animate<TAnimationKey>>,
) -> bool {
	let Some(animate) = animate else {
		return false;
	};

	animate == &skill.animate()
}

fn get_spawner(slots: &Slots, transforms: &Query<&GlobalTransform>) -> Option<Spawner> {
	let spawner_slot = slots.0.get(&SlotKey::SkillSpawn)?;
	let spawner_transform = transforms.get(spawner_slot.entity).ok()?;
	Some(Spawner(*spawner_transform))
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{Animate, Slot, SlotKey, WaitNext},
		traits::cast_update::{CastType, State},
	};
	use bevy::{
		ecs::component::Component,
		prelude::{App, Transform, Update, Vec3},
		time::{Real, Time},
	};
	use mockall::{mock, predicate::eq};
	use std::time::Duration;

	#[derive(PartialEq)]
	enum BehaviorOption {
		Run,
		Stop,
		Transform,
	}

	#[derive(PartialEq)]
	enum MockOption {
		BehaviorExecution(BehaviorOption),
		Animate,
	}

	#[derive(Debug, PartialEq, Clone, Copy)]
	enum _AnimationKey {
		A,
	}

	#[derive(Component)]
	struct _Skill {
		pub mock: Mock_Skill,
	}

	impl _Skill {
		pub fn without_default_setup_for<const N: usize>(no_setup: [MockOption; N]) -> Self {
			let mut mock = Mock_Skill::new();

			if !no_setup.contains(&MockOption::BehaviorExecution(BehaviorOption::Run)) {
				mock.expect_run().return_const(());
			}
			if !no_setup.contains(&MockOption::BehaviorExecution(BehaviorOption::Stop)) {
				mock.expect_stop().return_const(());
			}
			if !no_setup.contains(&MockOption::BehaviorExecution(BehaviorOption::Transform)) {
				mock.expect_apply_transform().return_const(());
			}
			if !no_setup.contains(&MockOption::Animate) {
				mock.expect_animate().return_const(Animate::None);
			}

			Self { mock }
		}
	}

	impl CastUpdate for _Skill {
		fn update(&mut self, delta: Duration) -> State {
			self.mock.update(delta)
		}
	}

	impl BehaviorExecution for _Skill {
		fn run(&self, agent: &mut EntityCommands, agent_transform: &Transform, spawner: &Spawner) {
			self.mock.run(agent, agent_transform, spawner)
		}

		fn stop(&self, agent: &mut EntityCommands) {
			self.mock.stop(agent)
		}

		fn apply_transform(&self, transform: &mut Transform, spawner: &Spawner) {
			self.mock.apply_transform(transform, spawner)
		}
	}

	impl GetAnimation<_AnimationKey> for _Skill {
		fn animate(&self) -> Animate<_AnimationKey> {
			self.mock.animate()
		}
	}

	mock! {
		_Skill {}
		impl CastUpdate for _Skill {
			fn update(&mut self, delta: Duration) -> State {}
		}
		impl BehaviorExecution for _Skill {
			fn run<'a, 'b, 'c>(&self, agent: &mut EntityCommands<'a, 'b, 'c>, agent_transform: &Transform, spawner: &Spawner) {}
			fn stop<'a, 'b, 'c>(&self, agent: &mut EntityCommands<'a, 'b, 'c>) {}
			fn apply_transform(&self, transform: &mut Transform, spawner: &Spawner) {}
		}
		impl GetAnimation<_AnimationKey> for _Skill {
			fn animate(&self) -> Animate<_AnimationKey> {}
		}
	}

	fn setup_app(skill_spawn_location: Vec3, agent_location: Vec3) -> (App, Entity) {
		let mut app = App::new();
		let mut time = Time::<Real>::default();

		let skill_spawner = app
			.world
			.spawn(GlobalTransform::from_translation(skill_spawn_location))
			.id();

		let agent = app
			.world
			.spawn((
				Slots(
					[(
						SlotKey::SkillSpawn,
						Slot {
							entity: skill_spawner,
							item: None,
							combo_skill: None,
						},
					)]
					.into(),
				),
				Transform::from_translation(agent_location),
			))
			.id();

		time.update();
		app.insert_resource(time);
		app.update();
		app.add_systems(Update, execute_skill::<_AnimationKey, _Skill, Real>);

		(app, agent)
	}

	fn tick_time(app: &mut App, delta: Duration) {
		let mut time = app.world.resource_mut::<Time<Real>>();
		let last_update = time.last_update().unwrap();
		time.update_with_instant(last_update + delta);
	}

	#[test]
	fn call_update_with_delta() {
		let (mut app, agent) = setup_app(Vec3::ZERO, Vec3::ZERO);
		let mut skill = _Skill::without_default_setup_for([]);

		skill
			.mock
			.expect_update()
			.times(1)
			.with(eq(Duration::from_millis(100)))
			.return_const(State::Casting(CastType::Pre));
		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default()));

		tick_time(&mut app, Duration::from_millis(100));
		app.update();
	}

	#[test]
	fn add_animation_when_new() {
		let (mut app, agent) = setup_app(Vec3::ZERO, Vec3::ZERO);
		let mut skill = _Skill::without_default_setup_for([]);

		skill.mock.expect_update().return_const(State::New);

		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default()));
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(Some(&Animate::None), agent.get::<Animate<_AnimationKey>>());
	}

	#[test]
	fn do_not_add_animate_when_not_new() {
		let (mut app, agent) = setup_app(Vec3::ZERO, Vec3::ZERO);
		let mut skill = _Skill::without_default_setup_for([]);

		skill
			.mock
			.expect_update()
			.return_const(State::Casting(CastType::Pre));

		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default()));
		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Animate<_AnimationKey>>());
	}

	#[test]
	fn add_marker_when_new_and_active() {
		let (mut app, agent) = setup_app(Vec3::ZERO, Vec3::ZERO);
		let mut skill = _Skill::without_default_setup_for([MockOption::Animate]);

		skill
			.mock
			.expect_update()
			.return_const(State::Activate(AgeType::New));
		skill
			.mock
			.expect_animate()
			.times(1)
			.return_const(Animate::Repeat(_AnimationKey::A));

		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default()));
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&Animate::Repeat(_AnimationKey::A)),
			agent.get::<Animate<_AnimationKey>>()
		);
	}

	#[test]
	fn remove_animation() {
		let (mut app, agent) = setup_app(Vec3::ZERO, Vec3::ZERO);
		let mut skill = _Skill::without_default_setup_for([MockOption::Animate]);

		skill.mock.expect_update().return_const(State::Done);
		skill
			.mock
			.expect_animate()
			.times(1)
			.return_const(Animate::Repeat(_AnimationKey::A));

		app.world.entity_mut(agent).insert((
			skill,
			Transform::default(),
			Animate::Repeat(_AnimationKey::A),
		));

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Animate<_AnimationKey>>());
	}

	#[test]
	fn do_not_remove_not_matching_animation() {
		let (mut app, agent) = setup_app(Vec3::ZERO, Vec3::ZERO);
		let mut skill = _Skill::without_default_setup_for([MockOption::Animate]);

		skill.mock.expect_update().return_const(State::Done);
		skill
			.mock
			.expect_animate()
			.times(1)
			.return_const(Animate::Replay(_AnimationKey::A));

		app.world.entity_mut(agent).insert((
			skill,
			Transform::default(),
			Animate::Repeat(_AnimationKey::A),
		));

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&Animate::Repeat(_AnimationKey::A)),
			agent.get::<Animate<_AnimationKey>>()
		);
	}

	#[test]
	fn do_not_remove_animate_when_not_done() {
		let (mut app, agent) = setup_app(Vec3::ZERO, Vec3::ZERO);
		let mut skill = _Skill::without_default_setup_for([MockOption::Animate]);

		skill
			.mock
			.expect_update()
			.return_const(State::Casting(CastType::After));
		skill
			.mock
			.expect_animate()
			.return_const(Animate::Replay(_AnimationKey::A));

		app.world.entity_mut(agent).insert((
			skill,
			Transform::default(),
			Animate::Replay(_AnimationKey::A),
		));

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&Animate::Replay(_AnimationKey::A)),
			agent.get::<Animate<_AnimationKey>>()
		);
	}

	#[test]
	fn remove_skill() {
		let (mut app, agent) = setup_app(Vec3::ZERO, Vec3::ZERO);
		let mut skill = _Skill::without_default_setup_for([]);

		skill.mock.expect_update().return_const(State::Done);

		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default()));

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<_Skill>());
	}

	#[test]
	fn do_not_remove_skill_when_not_done() {
		let (mut app, agent) = setup_app(Vec3::ZERO, Vec3::ZERO);
		let mut skill = _Skill::without_default_setup_for([]);

		skill
			.mock
			.expect_update()
			.return_const(State::Casting(CastType::After));

		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default()));

		app.update();

		let agent = app.world.entity(agent);

		assert!(agent.contains::<_Skill>());
	}

	#[test]
	fn add_wait_next() {
		let (mut app, agent) = setup_app(Vec3::ZERO, Vec3::ZERO);
		let mut skill = _Skill::without_default_setup_for([]);

		skill.mock.expect_update().return_const(State::Done);

		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default()));

		app.update();

		let agent = app.world.entity(agent);

		assert!(agent.contains::<WaitNext>());
	}

	#[test]
	fn do_not_add_wait_next_when_not_done() {
		let (mut app, agent) = setup_app(Vec3::ZERO, Vec3::ZERO);
		let mut skill = _Skill::without_default_setup_for([]);

		skill
			.mock
			.expect_update()
			.return_const(State::Casting(CastType::Pre));

		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default()));

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<WaitNext>());
	}

	#[test]
	fn remove_all_related_components_when_waiting_next() {
		let (mut app, agent) = setup_app(Vec3::ZERO, Vec3::ZERO);
		let mut skill = _Skill::without_default_setup_for([MockOption::Animate]);

		skill
			.mock
			.expect_update()
			.return_const(State::Casting(CastType::Pre));
		skill
			.mock
			.expect_animate()
			.times(1)
			.return_const(Animate::Repeat(_AnimationKey::A));

		app.world.entity_mut(agent).insert((
			skill,
			Transform::default(),
			Animate::Repeat(_AnimationKey::A),
			WaitNext,
		));

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			(false, false),
			(
				agent.contains::<_Skill>(),
				agent.contains::<Animate<_AnimationKey>>()
			)
		);
	}

	#[test]
	fn run() {
		let (mut app, agent) = setup_app(Vec3::new(1., 2., 3.), Vec3::new(3., 4., 5.));
		let mut skill =
			_Skill::without_default_setup_for([MockOption::BehaviorExecution(BehaviorOption::Run)]);

		skill
			.mock
			.expect_update()
			.return_const(State::Activate(AgeType::Old));
		skill
			.mock
			.expect_run()
			.times(1)
			.withf(move |a, a_t, s| {
				a.id() == agent
					&& *a_t == Transform::from_xyz(3., 4., 5.)
					&& s.0 == GlobalTransform::from_xyz(1., 2., 3.)
			})
			.return_const(());

		app.world.entity_mut(agent).insert(skill);

		app.update();
	}

	#[test]
	fn do_run_when_not_activating() {
		let (mut app, agent) = setup_app(Vec3::ZERO, Vec3::ZERO);
		let mut skill =
			_Skill::without_default_setup_for([MockOption::BehaviorExecution(BehaviorOption::Run)]);

		skill
			.mock
			.expect_update()
			.return_const(State::Casting(CastType::After));
		skill.mock.expect_run().times(0).return_const(());

		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default()));

		app.update();
	}

	#[test]
	fn run_when_new_active() {
		let (mut app, agent) = setup_app(Vec3::new(1., 2., 3.), Vec3::new(3., 4., 5.));
		let mut skill =
			_Skill::without_default_setup_for([MockOption::BehaviorExecution(BehaviorOption::Run)]);

		skill
			.mock
			.expect_update()
			.return_const(State::Activate(AgeType::New));
		skill
			.mock
			.expect_run()
			.times(1)
			.withf(move |a, a_t, s| {
				a.id() == agent
					&& *a_t == Transform::from_xyz(3., 4., 5.)
					&& s.0 == GlobalTransform::from_xyz(1., 2., 3.)
			})
			.return_const(());

		app.world.entity_mut(agent).insert(skill);

		app.update();
	}

	#[test]
	fn stop() {
		let (mut app, agent) = setup_app(Vec3::ZERO, Vec3::ZERO);
		let mut skill = _Skill::without_default_setup_for([MockOption::BehaviorExecution(
			BehaviorOption::Stop,
		)]);

		skill.mock.expect_update().return_const(State::Done);
		skill
			.mock
			.expect_stop()
			.times(1)
			.withf(move |a| a.id() == agent)
			.return_const(());

		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default()));

		app.update();
	}

	#[test]
	fn do_not_stop_when_not_done() {
		let (mut app, agent) = setup_app(Vec3::ZERO, Vec3::ZERO);
		let mut skill = _Skill::without_default_setup_for([MockOption::BehaviorExecution(
			BehaviorOption::Stop,
		)]);

		skill
			.mock
			.expect_update()
			.return_const(State::Casting(CastType::Pre));
		skill.mock.expect_stop().times(0).return_const(());

		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default()));

		app.update();
	}

	#[test]
	fn apply_transform() {
		let (mut app, agent) = setup_app(Vec3::new(11., 12., 13.), Vec3::ZERO);
		let mut skill = _Skill::without_default_setup_for([MockOption::BehaviorExecution(
			BehaviorOption::Transform,
		)]);

		let spawner = Spawner(GlobalTransform::from_xyz(11., 12., 13.));
		let transform = Transform::from_xyz(-1., -2., -3.);

		skill.mock.expect_update().return_const(State::New);
		skill
			.mock
			.expect_apply_transform()
			.times(1)
			.with(eq(transform), eq(spawner))
			.return_const(());

		app.world.entity_mut(agent).insert((skill, transform));

		app.update();
	}

	#[test]
	fn do_not_apply_transform_when_not_new() {
		let (mut app, agent) = setup_app(Vec3::ZERO, Vec3::ZERO);
		let mut skill = _Skill::without_default_setup_for([MockOption::BehaviorExecution(
			BehaviorOption::Transform,
		)]);

		skill
			.mock
			.expect_update()
			.return_const(State::Casting(CastType::Pre));
		skill
			.mock
			.expect_apply_transform()
			.times(0)
			.return_const(());

		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default()));

		app.update();
	}
}
