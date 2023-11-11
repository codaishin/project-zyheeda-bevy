use crate::components::{Skill, SlotKey, Slots, TimeTracker, WaitNext};
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
	With,
	Without,
};

type NewSkills<'a, TBehavior, TSpawnSlot> = (
	Entity,
	&'a Skill<TBehavior>,
	&'a mut Transform,
	&'a Slots<TSpawnSlot>,
);
type RunningSkills<'a, TBehavior> = (
	Entity,
	&'a Skill<TBehavior>,
	&'a mut TimeTracker<Skill<TBehavior>>,
);
type Skills<'a, TBehavior> = (Entity, &'a Skill<TBehavior>);

pub fn execute_skill<TBehavior: Send + Sync + 'static, TSpawnSlot: Send + Sync + 'static>(
	time: Res<Time<Real>>,
	mut commands: Commands,
	mut agents_with_new_skill: Query<
		NewSkills<TBehavior, TSpawnSlot>,
		Without<TimeTracker<Skill<TBehavior>>>,
	>,
	mut agents_with_running_skill: Query<RunningSkills<TBehavior>>,
	waiting_agents: Query<Skills<TBehavior>, With<WaitNext<TBehavior>>>,
	transforms: Query<&GlobalTransform>,
) {
	let delta = time.delta();

	for (agent, skill, mut transform, slots) in &mut agents_with_new_skill {
		initialize_and_track(
			&mut commands,
			agent,
			skill,
			&mut transform,
			slots,
			&transforms,
		);
	}

	for (agent, skill, mut tracker) in &mut agents_with_running_skill {
		update(&mut commands, agent, skill, &mut tracker, delta);
	}

	for (agent, skill) in &waiting_agents {
		remove_and_untrack(&mut commands, agent, skill);
	}
}

fn initialize_and_track<TBehavior: Send + Sync + 'static, TSpawnSlot: Send + Sync + 'static>(
	commands: &mut Commands,
	agent: Entity,
	skill: &Skill<TBehavior>,
	transform: &mut Transform,
	slots: &Slots<TSpawnSlot>,
	transforms: &Query<&GlobalTransform>,
) {
	let mut agent = commands.entity(agent);
	agent.insert(TimeTracker::<Skill<TBehavior>>::new());
	skill.animation.insert_marker_on(&mut agent);

	let Some(spawn_slot) = slots.0.get(&SlotKey::SkillSpawn) else {
		return;
	};

	let Ok(spawn) = transforms.get(spawn_slot.entity) else {
		return;
	};

	let Some(ray_length) = skill.ray.intersect_plane(spawn.translation(), Vec3::Y) else {
		return;
	};

	let target = skill.ray.origin + skill.ray.direction * ray_length;

	transform.look_at(
		Vec3::new(target.x, transform.translation.y, target.z),
		Vec3::Y,
	);
}

fn update<TBehavior: Send + Sync + 'static>(
	commands: &mut Commands,
	agent: Entity,
	skill: &Skill<TBehavior>,
	tracker: &mut TimeTracker<Skill<TBehavior>>,
	delta: std::time::Duration,
) {
	let mut agent = commands.entity(agent);

	tracker.duration += delta;

	if tracker.duration < skill.cast.pre + skill.cast.after {
		return;
	}

	agent.insert(WaitNext::<TBehavior>::new());
	agent.remove::<(Skill<TBehavior>, TimeTracker<Skill<TBehavior>>)>();
	skill.animation.remove_marker_on(&mut agent);
}

fn remove_and_untrack<TBehavior: Send + Sync + 'static>(
	commands: &mut Commands,
	agent: Entity,
	skill: &Skill<TBehavior>,
) {
	let mut agent = commands.entity(agent);
	agent.remove::<(Skill<TBehavior>, TimeTracker<Skill<TBehavior>>)>();
	skill.animation.remove_marker_on(&mut agent);
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{marker::Marker, Cast, Slot, SlotKey, WaitNext},
		test_tools::assert_eq_approx,
	};
	use bevy::{
		prelude::{App, Ray, Transform, Update, Vec3},
		time::{Real, Time},
	};
	use std::time::Duration;

	struct Tag;

	struct MockBehavior;

	const TEST_CAST: Cast = Cast {
		pre: Duration::from_millis(100),
		after: Duration::from_millis(100),
	};
	const TEST_RAY: Ray = Ray {
		origin: Vec3::Y,
		direction: Vec3::NEG_ONE,
	};

	fn setup_app(skill_spawn_location: Vec3) -> (App, Entity) {
		let mut app = App::new();
		let mut time = Time::<Real>::default();

		let skill_spawner = app
			.world
			.spawn(GlobalTransform::from_translation(skill_spawn_location))
			.id();

		let agent = app
			.world
			.spawn(Slots::<()>(
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
		app.add_systems(Update, execute_skill::<MockBehavior, ()>);

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
			Skill::<MockBehavior>::new(TEST_RAY, TEST_CAST, Marker::<Tag>::commands()),
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
			Skill::<MockBehavior>::new(
				TEST_RAY,
				Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				Marker::<Tag>::commands(),
			),
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
			Skill::<MockBehavior>::new(
				TEST_RAY,
				Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				Marker::<Tag>::commands(),
			),
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
			Skill::<MockBehavior>::new(
				TEST_RAY,
				Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				Marker::<Tag>::commands(),
			),
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
			Skill::<MockBehavior>::new(
				TEST_RAY,
				Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				Marker::<Tag>::commands(),
			),
			Transform::default(),
		));

		app.update();

		tick_time(&mut app, Duration::from_millis(700));

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			(false, false),
			(
				agent.contains::<Skill<MockBehavior>>(),
				agent.contains::<TimeTracker<Skill<MockBehavior>>>()
			)
		);
	}

	#[test]
	fn add_wait_next() {
		let (mut app, agent) = setup_app(Vec3::ZERO);
		app.world.entity_mut(agent).insert((
			Skill::<MockBehavior>::new(
				TEST_RAY,
				Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				Marker::<Tag>::commands(),
			),
			Transform::default(),
		));

		app.update();

		tick_time(&mut app, Duration::from_millis(700));

		app.update();

		let agent = app.world.entity(agent);

		assert!(agent.contains::<WaitNext<MockBehavior>>());
	}

	#[test]
	fn do_not_add_add_wait_next_too_early() {
		let (mut app, agent) = setup_app(Vec3::ZERO);
		app.world.entity_mut(agent).insert((
			Skill::<MockBehavior>::new(
				TEST_RAY,
				Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				Marker::<Tag>::commands(),
			),
			Transform::default(),
		));

		app.update();

		tick_time(&mut app, Duration::from_millis(699));

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<WaitNext<MockBehavior>>());
	}

	#[test]
	fn remove_all_related_components_when_waiting_next() {
		let (mut app, agent) = setup_app(Vec3::ZERO);
		app.world.entity_mut(agent).insert((
			Skill::<MockBehavior>::new(
				TEST_RAY,
				Cast {
					pre: Duration::from_millis(500),
					after: Duration::from_millis(200),
				},
				Marker::<Tag>::commands(),
			),
			Transform::default(),
		));

		app.update();

		app.world
			.entity_mut(agent)
			.insert(WaitNext::<MockBehavior>::new());

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			(false, false, false),
			(
				agent.contains::<Skill<MockBehavior>>(),
				agent.contains::<TimeTracker<Skill<MockBehavior>>>(),
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
			Skill::<MockBehavior>::new(ray, TEST_CAST, Marker::<Tag>::commands()),
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
			Skill::<MockBehavior>::new(ray, TEST_CAST, Marker::<Tag>::commands()),
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
			Skill::<MockBehavior>::new(ray, TEST_CAST, Marker::<Tag>::commands()),
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
			Skill::<MockBehavior>::new(ray, TEST_CAST, Marker::<Tag>::commands()),
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
}
