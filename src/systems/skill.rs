use crate::{
	behaviors::meta::{Agent, Spawner, StartBehaviorFn},
	components::{Queued, Skill, SlotKey, Slots, TimeTracker, WaitNext},
	errors::Error,
};
use bevy::prelude::{
	Commands,
	Entity,
	GlobalTransform,
	Query,
	Real,
	Res,
	Time,
	Transform,
	Without,
};

type Skills<'a> = (Entity, &'a Skill<Queued>, &'a mut Transform, &'a Slots);
type RunningSkills<'a> = (
	Entity,
	&'a mut Skill<Queued>,
	&'a mut TimeTracker<Skill<Queued>>,
	&'a Slots,
	Option<&'a WaitNext>,
);

enum SkillStatus {
	Inactive,
	Done,
	Running,
}

pub fn execute_skill(
	time: Res<Time<Real>>,
	mut commands: Commands,
	mut agents_with_new_skill: Query<Skills, Without<TimeTracker<Skill<Queued>>>>,
	mut agents_with_running_skill: Query<RunningSkills>,
	transforms: Query<&GlobalTransform>,
) -> Vec<Result<(), Error>> {
	let delta = time.delta();
	let mut results = Vec::new();

	for (entity, skill, mut transform, slots) in &mut agents_with_new_skill {
		let agent = Agent(entity);

		update_transform(&mut transform, skill, slots, &transforms);
		results.push(mark_agent_as_running(&mut commands, skill, agent));
	}

	for (entity, mut skill, mut tracker, slots, wait_next) in &mut agents_with_running_skill {
		let agent = Agent(entity);

		tracker.duration += delta;

		if let Some((spawner, run)) = can_trigger_skill(&skill, &tracker, slots, &transforms) {
			trigger_skill(&mut commands, &mut skill, agent, spawner, run);
		}

		results.push(match skill_status(&skill, &tracker, wait_next) {
			SkillStatus::Running => Ok(()),
			_ => mark_agent_as_done(&mut commands, &mut skill, agent),
		});
	}

	results
}

fn mark_agent_as_running(
	commands: &mut Commands,
	skill: &Skill<Queued>,
	agent: Agent,
) -> Result<(), Error> {
	let mut agent = commands.entity(agent.0);
	agent.insert(TimeTracker::<Skill<Queued>>::new());
	(skill.marker.insert_fn)(&mut agent, skill.data.slot)
}

fn mark_agent_as_done(
	commands: &mut Commands,
	skill: &mut Skill<Queued>,
	agent: Agent,
) -> Result<(), Error> {
	if let Some(stop) = skill.behavior.stop_fn {
		stop(commands, &agent)
	}

	let mut agent = commands.entity(agent.0);
	agent.insert(WaitNext);
	agent.remove::<(Skill<Queued>, TimeTracker<Skill<Queued>>)>();
	(skill.marker.remove_fn)(&mut agent, skill.data.slot)
}

fn update_transform(
	transform: &mut Transform,
	skill: &Skill<Queued>,
	slots: &Slots,
	transforms: &Query<&GlobalTransform>,
) {
	let Some(transform_fn) = skill.behavior.transform_fn else {
		return;
	};
	let Some(slot) = slots.0.get(&SlotKey::SkillSpawn) else {
		return;
	};
	let Ok(spawn_transform) = transforms.get(slot.entity) else {
		return;
	};

	transform_fn(transform, &Spawner(*spawn_transform), &skill.data.ray);
}

fn can_trigger_skill(
	skill: &Skill<Queued>,
	tracker: &TimeTracker<Skill<Queued>>,
	slots: &Slots,
	transforms: &Query<&GlobalTransform>,
) -> Option<(Spawner, StartBehaviorFn)> {
	if tracker.duration < skill.cast.pre {
		return None;
	}

	let spawner_slot = slots.0.get(&SlotKey::SkillSpawn)?;
	let spawner_transform = transforms.get(spawner_slot.entity).ok()?;

	Some((Spawner(*spawner_transform), skill.behavior.run_fn?))
}

fn trigger_skill(
	cmd: &mut Commands,
	skill: &mut Skill<Queued>,
	agent: Agent,
	spawner: Spawner,
	run: StartBehaviorFn,
) {
	skill.behavior.run_fn = None;

	run(cmd, &agent, &spawner, &skill.data.ray);
}

fn skill_status(
	skill: &Skill<Queued>,
	tracker: &TimeTracker<Skill<Queued>>,
	wait_next: Option<&WaitNext>,
) -> SkillStatus {
	if wait_next.is_some() {
		return SkillStatus::Inactive;
	}

	if tracker.duration >= skill.cast.pre + skill.cast.after {
		return SkillStatus::Done;
	}

	SkillStatus::Running
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		behaviors::meta::BehaviorMeta,
		components::{Cast, Marker, Side, Slot, SlotKey, WaitNext},
		errors::Level,
		markers::meta::MarkerMeta,
		systems::log::tests::{fake_log_error_lazy_many, FakeErrorLogMany},
		traits::{behavior::GetBehaviorMeta, marker::GetMarkerMeta},
	};
	use bevy::{
		ecs::{component::Component, system::IntoSystem},
		prelude::{App, Ray, Transform, Update, Vec3},
		time::{Real, Time},
		utils::default,
	};
	use mockall::{automock, predicate::eq};
	use std::time::Duration;

	type AgentEntity = Entity;
	type SpawnerEntity = Entity;

	struct Test;

	struct SideNone;

	struct SideLeft;

	struct SideRight;

	struct _Tools;

	#[automock]
	impl _Tools {
		pub fn _transform_fn(_transform: &mut Transform, _spawner: &Spawner, _ray: &Ray) {}
	}

	impl GetMarkerMeta for Test {
		fn marker() -> MarkerMeta {
			MarkerMeta {
				insert_fn: |entity, slot| {
					match slot {
						SlotKey::Hand(Side::Right) => entity.insert(Marker::<SideRight>::new()),
						SlotKey::Hand(Side::Left) => entity.insert(Marker::<SideLeft>::new()),
						_ => entity.insert(Marker::<SideNone>::new()),
					};
					Ok(())
				},
				remove_fn: |entity, slot| {
					match slot {
						SlotKey::Hand(Side::Right) => entity.remove::<Marker<SideRight>>(),
						SlotKey::Hand(Side::Left) => entity.remove::<Marker<SideLeft>>(),
						_ => entity.remove::<Marker<SideNone>>(),
					};
					Ok(())
				},
			}
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct MockBehavior {
		pub agent: Agent,
		pub ray: Ray,
		pub spawner: Spawner,
	}

	#[derive(Component, Debug, PartialEq)]
	struct MockIdle {
		pub agent: Agent,
	}

	const REAL_LAZY: BehaviorMeta = BehaviorMeta {
		run_fn: None,
		stop_fn: None,
		transform_fn: None,
	};

	impl GetBehaviorMeta for MockBehavior {
		fn behavior() -> BehaviorMeta {
			BehaviorMeta {
				run_fn: Some(|commands, agent, spawner, ray| {
					commands.spawn(MockBehavior {
						agent: *agent,
						ray: *ray,
						spawner: *spawner,
					});
				}),
				stop_fn: Some(|commands, agent| {
					commands.spawn(MockIdle { agent: *agent });
				}),
				transform_fn: None,
			}
		}
	}

	const TEST_CAST: Cast = Cast {
		pre: Duration::from_millis(100),
		after: Duration::from_millis(100),
	};
	const TEST_RAY: Ray = Ray {
		origin: Vec3::Y,
		direction: Vec3::NEG_ONE,
	};

	fn setup_app(skill_spawn_location: Vec3) -> (App, AgentEntity, SpawnerEntity) {
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
						skill: None,
					},
				)]
				.into(),
			))
			.id();

		time.update();
		app.insert_resource(time);
		app.update();
		app.add_systems(Update, execute_skill.pipe(fake_log_error_lazy_many(agent)));

		(app, agent, skill_spawner)
	}

	fn tick_time(app: &mut App, delta: Duration) {
		let mut time = app.world.resource_mut::<Time<Real>>();
		let last_update = time.last_update().unwrap();
		time.update_with_instant(last_update + delta);
	}

	#[test]
	fn add_marker() {
		let (mut app, agent, ..) = setup_app(Vec3::ZERO);
		app.world.entity_mut(agent).insert((
			Skill::<Queued> {
				name: "Some Skill",
				data: Queued {
					ray: TEST_RAY,
					slot: SlotKey::Hand(Side::Right),
				},
				cast: TEST_CAST,
				behavior: REAL_LAZY,
				marker: Test::marker(),
			},
			Transform::default(),
		));

		app.update();

		let agent = app.world.entity(agent);

		assert!(agent.contains::<Marker<SideRight>>());
	}

	#[test]
	fn return_add_marker_error() {
		let (mut app, agent, ..) = setup_app(Vec3::ZERO);
		app.world.entity_mut(agent).insert((
			Skill::<Queued> {
				name: "Some Skill",
				data: Queued {
					ray: TEST_RAY,
					slot: SlotKey::Hand(Side::Right),
				},
				cast: TEST_CAST,
				behavior: REAL_LAZY,
				marker: MarkerMeta {
					insert_fn: |_, _| {
						Err(Error {
							msg: "some message".to_owned(),
							lvl: Level::Warning,
						})
					},
					remove_fn: |_, _| Ok(()),
				},
			},
			Transform::default(),
		));

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
	fn remove_marker() {
		let (mut app, agent, ..) = setup_app(Vec3::ZERO);
		app.world.entity_mut(agent).insert((
			Skill::<Queued> {
				name: "Some Skill",
				data: Queued {
					ray: TEST_RAY,
					slot: SlotKey::Hand(Side::Right),
				},
				cast: Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				behavior: REAL_LAZY,
				marker: Test::marker(),
			},
			Transform::default(),
		));

		app.update();

		tick_time(&mut app, Duration::from_millis(700));

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Marker<SideLeft>>());
	}
	#[test]
	fn return_remove_marker_error() {
		let (mut app, agent, ..) = setup_app(Vec3::ZERO);
		app.world.entity_mut(agent).insert((
			Skill::<Queued> {
				name: "Some Skill",
				data: Queued {
					ray: TEST_RAY,
					slot: SlotKey::Hand(Side::Right),
				},
				cast: Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				behavior: REAL_LAZY,
				marker: MarkerMeta {
					insert_fn: |_, _| Ok(()),
					remove_fn: |_, _| {
						Err(Error {
							msg: "some message".to_owned(),
							lvl: Level::Warning,
						})
					},
				},
			},
			Transform::default(),
		));

		app.update();

		tick_time(&mut app, Duration::from_millis(700));

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
	fn do_not_remove_marker_after_insufficient_time() {
		let (mut app, agent, ..) = setup_app(Vec3::ZERO);
		app.world.entity_mut(agent).insert((
			Skill::<Queued> {
				name: "Some Skill",
				data: Queued {
					ray: TEST_RAY,
					slot: SlotKey::Hand(Side::Left),
				},
				cast: Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				behavior: REAL_LAZY,
				marker: Test::marker(),
			},
			Transform::default(),
		));

		app.update();

		tick_time(&mut app, Duration::from_millis(699));

		app.update();

		let agent = app.world.entity(agent);

		assert!(agent.contains::<Marker<SideLeft>>());
	}

	#[test]
	fn remove_marker_after_incremental_deltas() {
		let (mut app, agent, ..) = setup_app(Vec3::ZERO);
		app.world.entity_mut(agent).insert((
			Skill::<Queued> {
				name: "Some Skill",
				data: Queued {
					ray: TEST_RAY,
					slot: SlotKey::Hand(Side::Right),
				},
				cast: Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				behavior: REAL_LAZY,
				marker: Test::marker(),
			},
			Transform::default(),
		));

		app.update();

		tick_time(&mut app, Duration::from_millis(350));

		app.update();

		tick_time(&mut app, Duration::from_millis(350));

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Marker<SideLeft>>());
	}

	#[test]
	fn remove_skill_and_tracker() {
		let (mut app, agent, ..) = setup_app(Vec3::ZERO);
		app.world.entity_mut(agent).insert((
			Skill::<Queued> {
				data: Queued {
					ray: TEST_RAY,
					slot: SlotKey::Hand(Side::Right),
				},
				cast: Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				behavior: REAL_LAZY,
				..default()
			},
			Transform::default(),
		));

		app.update();

		tick_time(&mut app, Duration::from_millis(700));

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			(false, false),
			(
				agent.contains::<Skill>(),
				agent.contains::<TimeTracker<Skill>>()
			)
		);
	}

	#[test]
	fn add_wait_next() {
		let (mut app, agent, ..) = setup_app(Vec3::ZERO);
		app.world.entity_mut(agent).insert((
			Skill::<Queued> {
				data: Queued {
					ray: TEST_RAY,
					slot: SlotKey::Hand(Side::Right),
				},
				cast: Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				behavior: REAL_LAZY,
				..default()
			},
			Transform::default(),
		));

		app.update();

		tick_time(&mut app, Duration::from_millis(700));

		app.update();

		let agent = app.world.entity(agent);

		assert!(agent.contains::<WaitNext>());
	}

	#[test]
	fn do_not_add_add_wait_next_too_early() {
		let (mut app, agent, ..) = setup_app(Vec3::ZERO);
		app.world.entity_mut(agent).insert((
			Skill::<Queued> {
				data: Queued {
					ray: TEST_RAY,
					slot: SlotKey::Hand(Side::Right),
				},
				cast: Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				behavior: REAL_LAZY,
				..default()
			},
			Transform::default(),
		));

		app.update();

		tick_time(&mut app, Duration::from_millis(699));

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<WaitNext>());
	}

	#[test]
	fn remove_all_related_components_when_waiting_next() {
		let (mut app, agent, ..) = setup_app(Vec3::ZERO);
		app.world.entity_mut(agent).insert((
			Skill::<Queued> {
				data: Queued {
					ray: TEST_RAY,
					slot: SlotKey::Hand(Side::Right),
				},
				cast: Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				behavior: REAL_LAZY,
				..default()
			},
			Transform::default(),
		));

		app.update();

		app.world.entity_mut(agent).insert(WaitNext);

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			(false, false, false),
			(
				agent.contains::<Skill<Queued>>(),
				agent.contains::<TimeTracker<Skill<Queued>>>(),
				agent.contains::<Marker<SideLeft>>(),
			)
		);
	}

	#[test]
	fn start_behavior() {
		let (mut app, agent, ..) = setup_app(Vec3::ZERO);
		app.world.entity_mut(agent).insert((
			Skill::<Queued> {
				data: Queued {
					ray: TEST_RAY,
					slot: SlotKey::Hand(Side::Right),
				},
				cast: Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				behavior: MockBehavior::behavior(),
				..default()
			},
			Transform::default(),
		));

		app.update();

		tick_time(&mut app, Duration::from_millis(500));

		app.update();

		let behavior = app
			.world
			.iter_entities()
			.find_map(|e| e.get::<MockBehavior>());

		assert!(behavior.is_some());
	}

	#[test]
	fn stop_behavior() {
		let (mut app, agent, ..) = setup_app(Vec3::ZERO);
		app.world.entity_mut(agent).insert((
			Skill::<Queued> {
				data: Queued {
					ray: TEST_RAY,
					slot: SlotKey::Hand(Side::Right),
				},
				cast: Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				behavior: MockBehavior::behavior(),
				..default()
			},
			Transform::default(),
		));

		app.update();

		tick_time(&mut app, Duration::from_millis(500));

		app.update();

		tick_time(&mut app, Duration::from_millis(200));

		app.update();

		let idle = app.world.iter_entities().find_map(|e| e.get::<MockIdle>());

		assert_eq!(
			Some(&MockIdle {
				agent: Agent(agent)
			}),
			idle
		);
	}

	#[test]
	fn do_not_stop_behavior_before_skill_is_done() {
		let (mut app, agent, ..) = setup_app(Vec3::ZERO);
		app.world.entity_mut(agent).insert((
			Skill::<Queued> {
				data: Queued {
					ray: TEST_RAY,
					slot: SlotKey::Hand(Side::Right),
				},
				cast: Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				behavior: MockBehavior::behavior(),
				..default()
			},
			Transform::default(),
		));

		app.update();

		tick_time(&mut app, Duration::from_millis(500));

		app.update();

		tick_time(&mut app, Duration::from_millis(199));

		app.update();

		let idle = app.world.iter_entities().find_map(|e| e.get::<MockIdle>());

		assert!(idle.is_none());
	}

	#[test]
	fn not_spawned_before_pre_cast_behavior() {
		let (mut app, agent, ..) = setup_app(Vec3::ZERO);
		app.world.entity_mut(agent).insert((
			Skill::<Queued> {
				data: Queued {
					ray: TEST_RAY,
					slot: SlotKey::Hand(Side::Right),
				},
				cast: Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				behavior: MockBehavior::behavior(),
				..default()
			},
			Transform::default(),
		));

		app.update();

		tick_time(&mut app, Duration::from_millis(499));

		app.update();

		let behavior = app
			.world
			.iter_entities()
			.find_map(|e| e.get::<MockBehavior>());

		assert!(behavior.is_none());
	}

	#[test]
	fn not_spawned_multiple_times() {
		let (mut app, agent, ..) = setup_app(Vec3::ZERO);
		app.world.entity_mut(agent).insert((
			Skill::<Queued> {
				data: Queued {
					ray: TEST_RAY,
					slot: SlotKey::Hand(Side::Right),
				},
				cast: Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				behavior: MockBehavior::behavior(),
				..default()
			},
			Transform::default(),
		));

		app.update();

		tick_time(&mut app, Duration::from_millis(500));

		app.update();

		tick_time(&mut app, Duration::from_millis(1));

		app.update();

		let behaviors: Vec<_> = app
			.world
			.iter_entities()
			.filter_map(|e| e.get::<MockBehavior>())
			.collect();

		assert_eq!(1, behaviors.len());
	}

	#[test]
	fn not_spawned_multiple_times_with_not_perfectly_matching_deltas() {
		let (mut app, agent, ..) = setup_app(Vec3::ZERO);
		app.world.entity_mut(agent).insert((
			Skill::<Queued> {
				data: Queued {
					ray: TEST_RAY,
					slot: SlotKey::Hand(Side::Right),
				},
				cast: Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				behavior: MockBehavior::behavior(),
				..default()
			},
			Transform::default(),
		));

		app.update();

		tick_time(&mut app, Duration::from_millis(501));

		app.update();

		tick_time(&mut app, Duration::from_millis(1));

		app.update();

		let behaviors: Vec<_> = app
			.world
			.iter_entities()
			.filter_map(|e| e.get::<MockBehavior>())
			.collect();

		assert_eq!(1, behaviors.len());
	}

	#[test]
	fn spawn_behavior_with_proper_arguments() {
		let (mut app, agent, ..) = setup_app(Vec3::ONE);
		app.world.entity_mut(agent).insert((
			Skill::<Queued> {
				data: Queued {
					ray: TEST_RAY,
					slot: SlotKey::Hand(Side::Right),
				},
				cast: Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				behavior: MockBehavior::behavior(),
				..default()
			},
			Transform::default(),
		));

		app.update();

		tick_time(&mut app, Duration::from_millis(500));

		app.update();

		let behavior = app
			.world
			.iter_entities()
			.find_map(|e| e.get::<MockBehavior>());

		assert_eq!(
			Some(&MockBehavior {
				agent: Agent(agent),
				ray: TEST_RAY,
				spawner: Spawner(GlobalTransform::from_translation(Vec3::ONE)),
			}),
			behavior
		);
	}

	#[test]
	fn apply_transform_fn() {
		let (mut app, agent, spawner) = setup_app(Vec3::ONE);

		app.world.entity_mut(agent).insert((
			Skill::<Queued> {
				data: Queued {
					ray: TEST_RAY,
					slot: SlotKey::Hand(Side::Right),
				},
				cast: Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				behavior: BehaviorMeta {
					run_fn: None,
					stop_fn: None,
					transform_fn: Some(Mock_Tools::_transform_fn),
				},
				..default()
			},
			Transform::default(),
		));

		let spawner = Spawner(*app.world.entity(spawner).get::<GlobalTransform>().unwrap());
		let agent = *app.world.entity(agent).get::<Transform>().unwrap();
		let ctx = Mock_Tools::_transform_fn_context();
		ctx.expect()
			.once()
			.with(eq(agent), eq(spawner), eq(TEST_RAY))
			.return_const(());

		tick_time(&mut app, Duration::from_millis(100));

		app.update();
	}
}
