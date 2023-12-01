pub mod dequeue;
pub mod enqueue;
pub mod projectile;

use std::time::Duration;

use crate::{
	behaviors::meta::{Spawner, StartBehaviorFn, StopBehaviorFn},
	components::{Active, Skill, SlotKey, Slots, WaitNext},
	errors::Error,
};
use bevy::{
	ecs::system::EntityCommands,
	prelude::{Commands, Entity, GlobalTransform, Query, Real, Res, Time, Transform},
};

type Skills<'a> = (
	Entity,
	&'a mut Transform,
	&'a mut Skill<Active>,
	&'a Slots,
	Option<&'a WaitNext>,
);

pub fn execute_skill(
	time: Res<Time<Real>>,
	mut commands: Commands,
	mut agents: Query<Skills>,
	transforms: Query<&GlobalTransform>,
) -> Vec<Result<(), Error>> {
	let delta = time.delta();
	let mut results = Vec::new();

	for (entity, mut transform, mut skill, slots, wait_next) in &mut agents {
		let mut agent = commands.entity(entity);
		let progress = (delta, wait_next);
		results.push(execute_skill_on_agent(
			&mut agent,
			&mut skill,
			&mut transform,
			slots,
			&transforms,
			progress,
		));
	}

	results
}

fn execute_skill_on_agent(
	agent: &mut EntityCommands,
	skill: &mut Skill<Active>,
	transform: &mut Transform,
	slots: &Slots,
	transforms: &Query<&GlobalTransform>,
	progress: (Duration, Option<&WaitNext>),
) -> Result<(), Error> {
	if skill.data.duration.is_zero() {
		update_transform(transform, skill, slots, transforms);
		try_insert_markers(agent, skill)?;
	}

	skill.data.duration += progress.0;

	if let Some((spawner, run_skill)) = can_run_skill(skill, slots, transforms) {
		run_skill(agent, &spawner, &skill.data.ray);
		skill.behavior.run_fn = None;
	}

	if let Some(stop_skill) = can_stop_skill(skill, progress.1) {
		stop_skill(agent);
		agent.insert(WaitNext);
		agent.remove::<Skill<Active>>();
		try_remove_markers(agent, skill)?;
	}

	Ok(())
}

fn try_insert_markers(agent: &mut EntityCommands, skill: &mut Skill<Active>) -> Result<(), Error> {
	(skill.marker.insert_fn)(agent, skill.data.slot)
}

fn try_remove_markers(agent: &mut EntityCommands, skill: &mut Skill<Active>) -> Result<(), Error> {
	(skill.marker.remove_fn)(agent, skill.data.slot)
}

fn update_transform(
	transform: &mut Transform,
	skill: &Skill<Active>,
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

fn can_run_skill(
	skill: &Skill<Active>,
	slots: &Slots,
	transforms: &Query<&GlobalTransform>,
) -> Option<(Spawner, StartBehaviorFn)> {
	if skill.data.duration < skill.cast.pre {
		return None;
	}

	let spawner_slot = slots.0.get(&SlotKey::SkillSpawn)?;
	let spawner_transform = transforms.get(spawner_slot.entity).ok()?;

	Some((Spawner(*spawner_transform), skill.behavior.run_fn?))
}

fn can_stop_skill(skill: &Skill<Active>, wait_next: Option<&WaitNext>) -> Option<StopBehaviorFn> {
	if wait_next.is_none() && skill.data.duration < skill.cast.pre + skill.cast.after {
		return None;
	}

	Some(skill.behavior.stop_fn.unwrap_or(noop))
}

fn noop(_agent: &mut EntityCommands) {}

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
				soft_override: |_, _| false,
			}
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct MockBehavior {
		pub agent: Entity,
		pub ray: Ray,
		pub spawner: Spawner,
	}

	#[derive(Component, Debug, PartialEq)]
	struct MockIdle {
		pub agent: Entity,
	}

	const REAL_LAZY: BehaviorMeta = BehaviorMeta {
		run_fn: None,
		stop_fn: None,
		transform_fn: None,
	};
	const TEST_RAY: Ray = Ray {
		origin: Vec3::Y,
		direction: Vec3::NEG_ONE,
	};

	impl GetBehaviorMeta for MockBehavior {
		fn behavior() -> BehaviorMeta {
			BehaviorMeta {
				run_fn: Some(|agent, spawner, ray| {
					let id = agent.id();
					agent.commands().spawn(MockBehavior {
						agent: id,
						ray: *ray,
						spawner: *spawner,
					});
				}),
				stop_fn: Some(|agent| {
					let idle = MockIdle { agent: agent.id() };
					agent.commands().spawn(idle);
				}),
				transform_fn: None,
			}
		}
	}

	const TEST_CAST: Cast = Cast {
		pre: Duration::from_millis(100),
		after: Duration::from_millis(100),
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
			Skill::<Active> {
				name: "Some Skill",
				data: Active {
					slot: SlotKey::Hand(Side::Right),
					..default()
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
			Skill::<Active> {
				data: Active {
					slot: SlotKey::Hand(Side::Right),
					..default()
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
					soft_override: |_, _| false,
				},
				..default()
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
			Skill::<Active> {
				data: Active {
					slot: SlotKey::Hand(Side::Right),
					..default()
				},
				cast: Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				behavior: REAL_LAZY,
				marker: Test::marker(),
				..default()
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
			Skill::<Active> {
				data: Active {
					slot: SlotKey::Hand(Side::Right),
					..default()
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
					soft_override: |_, _| false,
				},
				..default()
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
			Skill::<Active> {
				data: Active {
					slot: SlotKey::Hand(Side::Left),
					..default()
				},
				cast: Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				behavior: REAL_LAZY,
				marker: Test::marker(),
				..default()
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
			Skill::<Active> {
				data: Active {
					slot: SlotKey::Hand(Side::Right),
					..default()
				},
				cast: Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				behavior: REAL_LAZY,
				marker: Test::marker(),
				..default()
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
	fn remove_skill() {
		let (mut app, agent, ..) = setup_app(Vec3::ZERO);
		app.world.entity_mut(agent).insert((
			Skill::<Active> {
				data: Active {
					slot: SlotKey::Hand(Side::Right),
					..default()
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

		assert!(!agent.contains::<Skill>());
	}

	#[test]
	fn add_wait_next() {
		let (mut app, agent, ..) = setup_app(Vec3::ZERO);
		app.world.entity_mut(agent).insert((
			Skill::<Active> {
				data: Active {
					slot: SlotKey::Hand(Side::Right),
					..default()
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
			Skill::<Active> {
				data: Active {
					slot: SlotKey::Hand(Side::Right),
					..default()
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
			Skill::<Active> {
				data: Active {
					slot: SlotKey::Hand(Side::Right),
					..default()
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
			(false, false),
			(
				agent.contains::<Skill<Active>>(),
				agent.contains::<Marker<SideLeft>>(),
			)
		);
	}

	#[test]
	fn start_behavior() {
		let (mut app, agent, ..) = setup_app(Vec3::ZERO);
		app.world.entity_mut(agent).insert((
			Skill::<Active> {
				data: Active {
					slot: SlotKey::Hand(Side::Right),
					..default()
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
			Skill::<Active> {
				data: Active {
					slot: SlotKey::Hand(Side::Right),
					..default()
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

		assert_eq!(Some(&MockIdle { agent }), idle);
	}

	#[test]
	fn do_not_stop_behavior_before_skill_is_done() {
		let (mut app, agent, ..) = setup_app(Vec3::ZERO);
		app.world.entity_mut(agent).insert((
			Skill::<Active> {
				data: Active {
					slot: SlotKey::Hand(Side::Right),
					..default()
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
			Skill::<Active> {
				data: Active {
					slot: SlotKey::Hand(Side::Right),
					..default()
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
			Skill::<Active> {
				data: Active {
					slot: SlotKey::Hand(Side::Right),
					..default()
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
			Skill::<Active> {
				data: Active {
					slot: SlotKey::Hand(Side::Right),
					..default()
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
			Skill::<Active> {
				data: Active {
					ray: TEST_RAY,
					slot: SlotKey::Hand(Side::Right),
					..default()
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
				agent,
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
			Skill::<Active> {
				data: Active {
					slot: SlotKey::Hand(Side::Right),
					ray: TEST_RAY,
					..default()
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
