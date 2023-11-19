use crate::components::{
	Agent,
	Skill,
	SlotKey,
	Slots,
	SpawnBehaviorFn,
	Spawner,
	TimeTracker,
	WaitNext,
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
	Vec3,
	Without,
};

type Skills<'a> = (Entity, &'a Skill, &'a mut Transform, &'a Slots);
type RunningSkills<'a> = (
	Entity,
	&'a mut Skill,
	&'a mut TimeTracker<Skill>,
	&'a Slots,
	Option<&'a WaitNext>,
);

pub fn execute_skill(
	time: Res<Time<Real>>,
	mut commands: Commands,
	mut agents_with_new_skill: Query<Skills, Without<TimeTracker<Skill>>>,
	mut agents_with_running_skill: Query<RunningSkills>,
	transforms: Query<&GlobalTransform>,
) {
	let delta = time.delta();

	for (entity, skill, mut transform, slots) in &mut agents_with_new_skill {
		let agent = Agent(entity);

		look_at_target(&mut transform, skill, slots, &transforms);
		mark_agent_as_running(&mut commands, skill, agent);
	}

	for (entity, mut skill, mut tracker, slots, wait_next) in &mut agents_with_running_skill {
		let agent = Agent(entity);

		tracker.duration += delta;

		if let Some((spawner, run)) = can_trigger_skill(&skill, &tracker, slots, &transforms) {
			trigger_skill(&mut commands, &mut skill, agent, spawner, run);
		}

		if skill_is_done(&skill, &tracker, wait_next) {
			mark_agent_as_done(&mut commands, &mut skill, agent);
		}
	}
}

fn mark_agent_as_running(commands: &mut Commands, skill: &Skill, agent: Agent) {
	let mut agent = commands.entity(agent.0);
	agent.insert(TimeTracker::<Skill>::new());
	skill.markers.insert_to(&mut agent);
}

fn mark_agent_as_done(commands: &mut Commands, skill: &mut Skill, agent: Agent) {
	if let Some(stop) = skill.behavior.stop_fn {
		stop(commands, agent)
	}

	let mut agent = commands.entity(agent.0);
	agent.insert(WaitNext);
	agent.remove::<(Skill, TimeTracker<Skill>)>();
	skill.markers.remove_from(&mut agent);
}

fn get_target(
	skill: &Skill,
	transform: &mut Transform,
	slots: &Slots,
	transforms: &Query<&GlobalTransform>,
) -> Option<Vec3> {
	let ray_length = slots
		.0
		.get(&SlotKey::SkillSpawn)
		.and_then(|slot| transforms.get(slot.entity).ok())
		.and_then(|entity| skill.ray.intersect_plane(entity.translation(), Vec3::Y))?;
	let target = skill.ray.origin + skill.ray.direction * ray_length;

	Some(Vec3::new(target.x, transform.translation.y, target.z))
}

fn look_at_target(
	transform: &mut Transform,
	skill: &Skill,
	slots: &Slots,
	transforms: &Query<&GlobalTransform>,
) {
	let Some(target) = get_target(skill, transform, slots, transforms) else {
		return;
	};

	transform.look_at(target, Vec3::Y);
}

fn can_trigger_skill(
	skill: &Skill,
	tracker: &TimeTracker<Skill>,
	slots: &Slots,
	transforms: &Query<&GlobalTransform>,
) -> Option<(Spawner, SpawnBehaviorFn)> {
	if tracker.duration < skill.cast.pre {
		return None;
	}

	let spawner_slot = slots.0.get(&SlotKey::SkillSpawn)?;
	let spawner_transform = transforms.get(spawner_slot.entity).ok()?;

	Some((Spawner(*spawner_transform), skill.behavior.run_fn?))
}

fn trigger_skill(
	cmd: &mut Commands,
	skill: &mut Skill,
	agent: Agent,
	spawner: Spawner,
	run: SpawnBehaviorFn,
) {
	skill.behavior.run_fn = None;

	run(cmd, agent, spawner, skill.ray);
}

fn skill_is_done(
	skill: &Skill,
	tracker: &TimeTracker<Skill>,
	wait_next: Option<&WaitNext>,
) -> bool {
	wait_next.is_some() || tracker.duration >= skill.cast.pre + skill.cast.after
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{lazy::Lazy, marker::Marker, Cast, Slot, SlotKey, WaitNext},
		test_tools::assert_eq_approx,
		traits::to_lazy::ToLazy,
	};
	use bevy::{
		ecs::component::Component,
		prelude::{App, Ray, Transform, Update, Vec3},
		time::{Real, Time},
	};
	use std::time::Duration;

	type AgentEntity = Entity;

	struct Tag;

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

	const REAL_LAZY: Lazy = Lazy {
		run_fn: None,
		stop_fn: None,
	};

	impl ToLazy for MockBehavior {
		fn to_lazy() -> Lazy {
			Lazy {
				run_fn: Some(|commands, agent, spawner, ray| {
					commands.spawn(MockBehavior {
						agent,
						ray,
						spawner,
					});
				}),
				stop_fn: Some(|commands, agent| {
					commands.spawn(MockIdle { agent });
				}),
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

	fn setup_app(skill_spawn_location: Vec3) -> (App, AgentEntity) {
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
						behavior: None,
					},
				)]
				.into(),
			))
			.id();

		time.update();
		app.insert_resource(time);
		app.update();
		app.add_systems(Update, execute_skill);

		(app, agent)
	}

	fn tick_time(app: &mut App, delta: Duration) {
		let mut time = app.world.resource_mut::<Time<Real>>();
		let last_update = time.last_update().unwrap();
		time.update_with_instant(last_update + delta);
	}

	#[test]
	fn add_marker() {
		let (mut app, agent) = setup_app(Vec3::ZERO);
		app.world.entity_mut(agent).insert((
			Skill {
				ray: TEST_RAY,
				cast: TEST_CAST,
				markers: Marker::<Tag>::commands(),
				behavior: REAL_LAZY,
			},
			Transform::default(),
		));

		app.update();

		let agent = app.world.entity(agent);

		assert!(agent.contains::<Marker<Tag>>());
	}

	#[test]
	fn remove_marker() {
		let (mut app, agent) = setup_app(Vec3::ZERO);
		app.world.entity_mut(agent).insert((
			Skill {
				ray: TEST_RAY,
				cast: Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				markers: Marker::<Tag>::commands(),
				behavior: REAL_LAZY,
			},
			Transform::default(),
		));

		app.update();

		tick_time(&mut app, Duration::from_millis(700));

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Marker<Tag>>());
	}

	#[test]
	fn do_not_remove_marker_after_insufficient_time() {
		let (mut app, agent) = setup_app(Vec3::ZERO);
		app.world.entity_mut(agent).insert((
			Skill {
				ray: TEST_RAY,
				cast: Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				markers: Marker::<Tag>::commands(),
				behavior: REAL_LAZY,
			},
			Transform::default(),
		));

		app.update();

		tick_time(&mut app, Duration::from_millis(699));

		app.update();

		let agent = app.world.entity(agent);

		assert!(agent.contains::<Marker<Tag>>());
	}

	#[test]
	fn remove_marker_after_incremental_deltas() {
		let (mut app, agent) = setup_app(Vec3::ZERO);
		app.world.entity_mut(agent).insert((
			Skill {
				ray: TEST_RAY,
				cast: Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				markers: Marker::<Tag>::commands(),
				behavior: REAL_LAZY,
			},
			Transform::default(),
		));

		app.update();

		tick_time(&mut app, Duration::from_millis(350));

		app.update();

		tick_time(&mut app, Duration::from_millis(350));

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Marker<Tag>>());
	}

	#[test]
	fn remove_skill_and_tracker() {
		let (mut app, agent) = setup_app(Vec3::ZERO);
		app.world.entity_mut(agent).insert((
			Skill {
				ray: TEST_RAY,
				cast: Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				markers: Marker::<Tag>::commands(),
				behavior: REAL_LAZY,
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
		let (mut app, agent) = setup_app(Vec3::ZERO);
		app.world.entity_mut(agent).insert((
			Skill {
				ray: TEST_RAY,
				cast: Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				markers: Marker::<Tag>::commands(),
				behavior: REAL_LAZY,
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
		let (mut app, agent) = setup_app(Vec3::ZERO);
		app.world.entity_mut(agent).insert((
			Skill {
				ray: TEST_RAY,
				cast: Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				markers: Marker::<Tag>::commands(),
				behavior: REAL_LAZY,
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
		let (mut app, agent) = setup_app(Vec3::ZERO);
		app.world.entity_mut(agent).insert((
			Skill {
				ray: TEST_RAY,
				cast: Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				markers: Marker::<Tag>::commands(),
				behavior: REAL_LAZY,
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
				agent.contains::<Skill>(),
				agent.contains::<TimeTracker<Skill>>(),
				agent.contains::<Marker<Tag>>(),
			)
		);
	}

	#[test]
	fn use_ray_look_direction() {
		let (mut app, agent) = setup_app(Vec3::ZERO);
		let ray = Ray {
			origin: Vec3::new(1., 10., 5.),
			direction: Vec3::NEG_Y,
		};
		app.world.entity_mut(agent).insert((
			Skill {
				ray,
				cast: TEST_CAST,
				markers: Marker::<Tag>::commands(),
				behavior: REAL_LAZY,
			},
			Transform::default(),
		));

		app.update();

		let agent = app.world.entity(agent);
		let agent = agent.get::<Transform>().unwrap();

		assert_eq_approx(Vec3::new(1., 0., 5.).normalize(), agent.forward(), 0.000001);
	}

	#[test]
	fn use_odd_ray_look_direction() {
		let (mut app, agent) = setup_app(Vec3::ZERO);
		let ray = Ray {
			origin: Vec3::new(0., 3., 0.),
			direction: Vec3::new(4., -3., 0.),
		};
		app.world.entity_mut(agent).insert((
			Skill {
				ray,
				cast: TEST_CAST,
				markers: Marker::<Tag>::commands(),
				behavior: REAL_LAZY,
			},
			Transform::default(),
		));

		app.update();

		let agent = app.world.entity(agent);
		let agent = agent.get::<Transform>().unwrap();

		assert_eq_approx(
			(Vec3::new(5., 0., 0.)).normalize(),
			agent.forward(),
			0.000001,
		);
	}

	#[test]
	fn use_odd_ray_and_skill_spawn_for_look_direction() {
		let (mut app, agent) = setup_app(Vec3::new(0., 3., 0.));
		let ray = Ray {
			origin: Vec3::new(0., 6., 0.),
			direction: Vec3::new(4., -3., 0.),
		};
		app.world.entity_mut(agent).insert((
			Skill {
				ray,
				cast: TEST_CAST,
				markers: Marker::<Tag>::commands(),
				behavior: REAL_LAZY,
			},
			Transform::from_xyz(0., 3., 0.),
		));

		app.update();

		let agent = app.world.entity(agent);
		let agent = agent.get::<Transform>().unwrap();

		assert_eq_approx(
			(Vec3::new(5., 0., 0.)).normalize(),
			agent.forward(),
			0.000001,
		);
	}

	#[test]
	fn look_horizontally() {
		let (mut app, agent) = setup_app(Vec3::new(0., 3., 0.));
		let ray = Ray {
			origin: Vec3::new(0., 6., 0.),
			direction: Vec3::new(4., -3., 0.),
		};
		app.world.entity_mut(agent).insert((
			Skill {
				ray,
				cast: TEST_CAST,
				markers: Marker::<Tag>::commands(),
				behavior: REAL_LAZY,
			},
			Transform::from_xyz(0., 0., 0.),
		));

		app.update();

		let agent = app.world.entity(agent);
		let agent = agent.get::<Transform>().unwrap();

		assert_eq_approx(
			(Vec3::new(5., 0., 0.)).normalize(),
			agent.forward(),
			0.000001,
		);
	}

	#[test]
	fn start_behavior() {
		let (mut app, agent) = setup_app(Vec3::ZERO);
		app.world.entity_mut(agent).insert((
			Skill {
				ray: TEST_RAY,
				cast: Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				markers: Marker::<Tag>::commands(),
				behavior: MockBehavior::to_lazy(),
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
		let (mut app, agent) = setup_app(Vec3::ZERO);
		app.world.entity_mut(agent).insert((
			Skill {
				ray: TEST_RAY,
				cast: Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				markers: Marker::<Tag>::commands(),
				behavior: MockBehavior::to_lazy(),
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
		let (mut app, agent) = setup_app(Vec3::ZERO);
		app.world.entity_mut(agent).insert((
			Skill {
				ray: TEST_RAY,
				cast: Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				markers: Marker::<Tag>::commands(),
				behavior: MockBehavior::to_lazy(),
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
		let (mut app, agent) = setup_app(Vec3::ZERO);
		app.world.entity_mut(agent).insert((
			Skill {
				ray: TEST_RAY,
				cast: Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				markers: Marker::<Tag>::commands(),
				behavior: MockBehavior::to_lazy(),
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
		let (mut app, agent) = setup_app(Vec3::ZERO);
		app.world.entity_mut(agent).insert((
			Skill {
				ray: TEST_RAY,
				cast: Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				markers: Marker::<Tag>::commands(),
				behavior: MockBehavior::to_lazy(),
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
		let (mut app, agent) = setup_app(Vec3::ZERO);
		app.world.entity_mut(agent).insert((
			Skill {
				ray: TEST_RAY,
				cast: Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				markers: Marker::<Tag>::commands(),
				behavior: MockBehavior::to_lazy(),
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
		let (mut app, agent) = setup_app(Vec3::ONE);
		app.world.entity_mut(agent).insert((
			Skill {
				ray: TEST_RAY,
				cast: Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				markers: Marker::<Tag>::commands(),
				behavior: MockBehavior::to_lazy(),
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
}
