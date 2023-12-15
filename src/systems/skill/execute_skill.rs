use crate::{
	behaviors::meta::Spawner,
	components::{SlotKey, Slots, WaitNext},
	errors::Error,
	traits::{
		behavior_execution::BehaviorExecution,
		cast_update::{CastUpdate, State},
		marker_modify::MarkerModify,
	},
};
use bevy::{
	ecs::{component::Component, system::EntityCommands},
	prelude::{Commands, Entity, GlobalTransform, Mut, Query, Res, Time, Transform},
};
use std::time::Duration;

type Skills<'a, TSkill> = (
	Entity,
	&'a mut Transform,
	&'a mut TSkill,
	&'a Slots,
	Option<&'a WaitNext>,
);

pub fn execute_skill<
	TSkill: CastUpdate + MarkerModify + BehaviorExecution + Component,
	TTime: Send + Sync + Default + 'static,
>(
	time: Res<Time<TTime>>,
	mut commands: Commands,
	mut agents: Query<Skills<TSkill>>,
	transforms: Query<&GlobalTransform>,
) -> Vec<Result<(), Error>> {
	let delta = time.delta();
	let handle_skill = |(entity, mut transform, mut skill, slots, wait_next)| {
		let agent = &mut commands.entity(entity);
		let transform = &mut transform;
		let skill = &mut skill;
		let transforms = &transforms;

		match get_state(skill, &delta, wait_next) {
			State::New => handle_new(agent, transform, skill, slots, transforms),
			State::Activate => handle_active(agent, skill, slots, transforms),
			State::Done => handle_done(agent, skill),
			_ => Ok(()),
		}
	};

	agents.iter_mut().map(handle_skill).collect()
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

fn handle_new<TSkill: MarkerModify + BehaviorExecution>(
	agent: &mut EntityCommands,
	transform: &mut Mut<Transform>,
	skill: &mut Mut<TSkill>,
	slots: &Slots,
	transforms: &Query<&GlobalTransform>,
) -> Result<(), Error> {
	if let Some(spawner) = get_spawner(slots, transforms) {
		skill.apply_transform(transform, &spawner);
	};
	skill.insert_markers(agent)
}

fn handle_active<TSkill: BehaviorExecution>(
	agent: &mut EntityCommands,
	skill: &mut Mut<TSkill>,
	slots: &Slots,
	transforms: &Query<&GlobalTransform>,
) -> Result<(), Error> {
	if let Some(spawner) = get_spawner(slots, transforms) {
		skill.run(agent, &spawner);
	};
	Ok(())
}

fn handle_done<TSkill: CastUpdate + MarkerModify + BehaviorExecution + Component>(
	agent: &mut EntityCommands,
	skill: &mut Mut<TSkill>,
) -> Result<(), Error> {
	agent.remove::<TSkill>();
	agent.insert(WaitNext);
	skill.stop(agent);
	skill.remove_markers(agent)
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
		components::{Slot, SlotKey, WaitNext},
		errors::Level,
		systems::log::tests::{fake_log_error_lazy_many, FakeErrorLogMany},
		traits::cast_update::{CastType, State},
	};
	use bevy::{
		ecs::{component::Component, system::IntoSystem},
		prelude::{App, Transform, Update, Vec3},
		time::{Real, Time},
	};
	use mockall::{mock, predicate::eq};
	use std::time::Duration;

	#[derive(PartialEq)]
	enum MarkerOption {
		Insert,
		Remove,
	}

	#[derive(PartialEq)]
	enum BehaviorOption {
		Run,
		Stop,
		Transform,
	}

	#[derive(PartialEq)]
	enum MockOption {
		MarkerModify(MarkerOption),
		BehaviorExecution(BehaviorOption),
	}

	#[derive(Component)]
	struct _Skill {
		pub mock: Mock_Skill,
	}

	impl _Skill {
		pub fn without_default_setup_for<const N: usize>(no_setup: [MockOption; N]) -> Self {
			let mut mock = Mock_Skill::new();

			if !no_setup.contains(&MockOption::MarkerModify(MarkerOption::Insert)) {
				mock.expect_insert_markers().return_const(Ok(()));
			}
			if !no_setup.contains(&MockOption::MarkerModify(MarkerOption::Remove)) {
				mock.expect_remove_markers().return_const(Ok(()));
			}
			if !no_setup.contains(&MockOption::BehaviorExecution(BehaviorOption::Run)) {
				mock.expect_run().return_const(());
			}
			if !no_setup.contains(&MockOption::BehaviorExecution(BehaviorOption::Stop)) {
				mock.expect_stop().return_const(());
			}
			if !no_setup.contains(&MockOption::BehaviorExecution(BehaviorOption::Transform)) {
				mock.expect_apply_transform().return_const(());
			}

			Self { mock }
		}
	}

	impl CastUpdate for _Skill {
		fn update(&mut self, delta: Duration) -> State {
			self.mock.update(delta)
		}
	}

	impl MarkerModify for _Skill {
		fn insert_markers(&self, agent: &mut EntityCommands) -> Result<(), Error> {
			self.mock.insert_markers(agent)
		}

		fn remove_markers(&self, agent: &mut EntityCommands) -> Result<(), Error> {
			self.mock.remove_markers(agent)
		}
	}

	impl BehaviorExecution for _Skill {
		fn run(&self, agent: &mut EntityCommands, spawner: &Spawner) {
			self.mock.run(agent, spawner)
		}

		fn stop(&self, agent: &mut EntityCommands) {
			self.mock.stop(agent)
		}

		fn apply_transform(&self, transform: &mut Transform, spawner: &Spawner) {
			self.mock.apply_transform(transform, spawner)
		}
	}

	mock! {
		_Skill {}
		impl CastUpdate for _Skill {
			fn update(&mut self, delta: Duration) -> State {
				State::Done
			}
		}
		impl MarkerModify for _Skill {
			fn insert_markers<'a, 'b, 'c>(&self, agent: &mut EntityCommands<'a, 'b, 'c>) -> Result<(), Error> {
				Ok(())
			}
			fn remove_markers<'a, 'b, 'c>(&self, agent: &mut EntityCommands<'a, 'b, 'c>) -> Result<(), Error>{
				Ok(())
			}
		}
		impl BehaviorExecution for _Skill {
			fn run<'a, 'b, 'c>(&self, agent: &mut EntityCommands<'a, 'b, 'c>, spawner: &Spawner) {
				()
			}
			fn stop<'a, 'b, 'c>(&self, agent: &mut EntityCommands<'a, 'b, 'c>) {
				()
			}
			fn apply_transform(&self, transform: &mut Transform, spawner: &Spawner) {
				()
			}
		}
	}

	fn setup_app(skill_spawn_location: Vec3) -> (App, Entity) {
		let mut app = App::new();
		let mut time = Time::<Real>::default();

		let skill_spawner = app
			.world
			.spawn(GlobalTransform::from_translation(skill_spawn_location))
			.id();

		let agent = app
			.world
			.spawn(Slots(
				[(
					SlotKey::SkillSpawn,
					Slot {
						entity: skill_spawner,
						item: None,
					},
				)]
				.into(),
			))
			.id();

		time.update();
		app.insert_resource(time);
		app.update();
		app.add_systems(
			Update,
			execute_skill::<_Skill, Real>.pipe(fake_log_error_lazy_many(agent)),
		);

		(app, agent)
	}

	fn tick_time(app: &mut App, delta: Duration) {
		let mut time = app.world.resource_mut::<Time<Real>>();
		let last_update = time.last_update().unwrap();
		time.update_with_instant(last_update + delta);
	}

	#[test]
	fn call_update_with_delta() {
		let (mut app, agent) = setup_app(Vec3::ZERO);
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
	fn add_marker() {
		let (mut app, agent) = setup_app(Vec3::ZERO);
		let mut skill =
			_Skill::without_default_setup_for([MockOption::MarkerModify(MarkerOption::Insert)]);

		skill.mock.expect_update().return_const(State::New);
		skill
			.mock
			.expect_insert_markers()
			.times(1)
			.withf(move |a| a.id() == agent)
			.return_const(Ok(()));

		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default()));
		app.update();
	}

	#[test]
	fn return_add_marker_error() {
		let (mut app, agent) = setup_app(Vec3::ZERO);
		let mut skill =
			_Skill::without_default_setup_for([MockOption::MarkerModify(MarkerOption::Insert)]);

		skill.mock.expect_update().return_const(State::New);
		skill.mock.expect_insert_markers().return_const(Err(Error {
			msg: "some message".to_owned(),
			lvl: Level::Warning,
		}));

		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default()));

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&FakeErrorLogMany(
				[Error {
					msg: "some message".to_owned(),
					lvl: Level::Warning
				}]
				.into()
			)),
			agent.get::<FakeErrorLogMany>()
		)
	}

	#[test]
	fn do_not_add_marker_when_not_new() {
		let (mut app, agent) = setup_app(Vec3::ZERO);
		let mut skill =
			_Skill::without_default_setup_for([MockOption::MarkerModify(MarkerOption::Insert)]);

		skill
			.mock
			.expect_update()
			.return_const(State::Casting(CastType::Pre));
		skill
			.mock
			.expect_insert_markers()
			.times(0)
			.return_const(Ok(()));

		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default()));
		app.update();
	}

	#[test]
	fn remove_marker() {
		let (mut app, agent) = setup_app(Vec3::ZERO);
		let mut skill =
			_Skill::without_default_setup_for([MockOption::MarkerModify(MarkerOption::Remove)]);

		skill.mock.expect_update().return_const(State::Done);
		skill
			.mock
			.expect_remove_markers()
			.times(1)
			.withf(move |a| a.id() == agent)
			.return_const(Ok(()));

		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default()));

		app.update();
	}

	#[test]
	fn remove_marker_error() {
		let (mut app, agent) = setup_app(Vec3::ZERO);
		let mut skill =
			_Skill::without_default_setup_for([MockOption::MarkerModify(MarkerOption::Remove)]);

		skill.mock.expect_update().return_const(State::Done);
		skill.mock.expect_remove_markers().return_const(Err(Error {
			msg: "some message".to_owned(),
			lvl: Level::Warning,
		}));

		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default()));

		app.update();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&FakeErrorLogMany(
				[Error {
					msg: "some message".to_owned(),
					lvl: Level::Warning
				}]
				.into()
			)),
			agent.get::<FakeErrorLogMany>()
		)
	}

	#[test]
	fn do_not_remove_marker_when_not_done() {
		let (mut app, agent) = setup_app(Vec3::ZERO);
		let mut skill =
			_Skill::without_default_setup_for([MockOption::MarkerModify(MarkerOption::Remove)]);

		skill
			.mock
			.expect_update()
			.return_const(State::Casting(CastType::After));
		skill
			.mock
			.expect_remove_markers()
			.times(0)
			.return_const(Ok(()));

		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default()));

		app.update();
	}

	#[test]
	fn remove_skill() {
		let (mut app, agent) = setup_app(Vec3::ZERO);
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
		let (mut app, agent) = setup_app(Vec3::ZERO);
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
		let (mut app, agent) = setup_app(Vec3::ZERO);
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
		let (mut app, agent) = setup_app(Vec3::ZERO);
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
		let (mut app, agent) = setup_app(Vec3::ZERO);
		let mut skill =
			_Skill::without_default_setup_for([MockOption::MarkerModify(MarkerOption::Remove)]);

		skill
			.mock
			.expect_update()
			.return_const(State::Casting(CastType::Pre));
		skill
			.mock
			.expect_remove_markers()
			.times(1)
			.withf(move |a| a.id() == agent)
			.return_const(Ok(()));

		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default(), WaitNext));

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<_Skill>());
	}

	#[test]
	fn done_works_even_with_remove_marker_error() {
		let (mut app, agent) = setup_app(Vec3::ZERO);
		let mut skill = _Skill::without_default_setup_for([
			MockOption::MarkerModify(MarkerOption::Remove),
			MockOption::BehaviorExecution(BehaviorOption::Stop),
		]);

		skill.mock.expect_update().return_const(State::Done);
		skill.mock.expect_remove_markers().return_const(Err(Error {
			msg: "".to_owned(),
			lvl: Level::Error,
		}));
		skill.mock.expect_stop().times(1).return_const(());

		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default(), WaitNext));

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			(false, true),
			(agent.contains::<_Skill>(), agent.contains::<WaitNext>())
		);
	}

	#[test]
	fn run() {
		let (mut app, agent) = setup_app(Vec3::new(1., 2., 3.));
		let mut skill =
			_Skill::without_default_setup_for([MockOption::BehaviorExecution(BehaviorOption::Run)]);

		skill.mock.expect_update().return_const(State::Activate);
		skill
			.mock
			.expect_run()
			.times(1)
			.withf(move |a, s| a.id() == agent && s.0 == GlobalTransform::from_xyz(1., 2., 3.))
			.return_const(());

		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default()));

		app.update();
	}

	#[test]
	fn do_run_when_not_activating() {
		let (mut app, agent) = setup_app(Vec3::ZERO);
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
	fn stop() {
		let (mut app, agent) = setup_app(Vec3::ZERO);
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
		let (mut app, agent) = setup_app(Vec3::ZERO);
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
		let (mut app, agent) = setup_app(Vec3::new(11., 12., 13.));
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
		let (mut app, agent) = setup_app(Vec3::ZERO);
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

	#[test]
	fn apply_transform_even_with_insert_marker_error() {
		let (mut app, agent) = setup_app(Vec3::ZERO);
		let mut skill = _Skill::without_default_setup_for([
			MockOption::MarkerModify(MarkerOption::Insert),
			MockOption::BehaviorExecution(BehaviorOption::Transform),
		]);

		skill.mock.expect_update().return_const(State::New);
		skill.mock.expect_insert_markers().return_const(Err(Error {
			msg: "".to_owned(),
			lvl: Level::Error,
		}));
		skill
			.mock
			.expect_apply_transform()
			.times(1)
			.return_const(());

		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default()));

		app.update();
	}
}
