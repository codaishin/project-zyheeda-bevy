use std::time::Duration;

use crate::components::{Skill, SlotKey, Slots, TimeTracker, WaitNext};
use bevy::{
	ecs::system::EntityCommands,
	prelude::{
		Commands,
		Entity,
		GlobalTransform,
		Mut,
		Query,
		Real,
		Res,
		Time,
		Transform,
		Vec3,
		Without,
	},
};

type NewSkills<'a> = (Entity, &'a Skill, &'a mut Transform, &'a Slots);
type RunningSkills<'a> = (
	Entity,
	&'a Skill,
	&'a mut TimeTracker<Skill>,
	Option<&'a WaitNext>,
);
type TrackedSkills<'a> = (
	Entity,
	&'a Skill,
	Mut<'a, TimeTracker<Skill>>,
	Option<&'a WaitNext>,
);

pub fn execute_skill(
	time: Res<Time<Real>>,
	mut commands: Commands,
	mut agents_with_new_skill: Query<NewSkills, Without<TimeTracker<Skill>>>,
	mut agents_with_running_skill: Query<RunningSkills>,
	transforms: Query<&GlobalTransform>,
) {
	for (agent, skill, mut transform, slots) in &mut agents_with_new_skill {
		let mut agent = commands.entity(agent);
		insert_components_to(&mut agent, skill);
		look_at_target(&mut transform, skill, slots, &transforms);
	}

	let agents_with_done_skill = agents_with_running_skill
		.iter_mut()
		.map(update_duration_with(time.delta()))
		.filter(skill_is_done);

	for (agent, skill, ..) in agents_with_done_skill {
		let mut agent = commands.entity(agent);
		remove_components_from(&mut agent, skill);
		agent.insert(WaitNext);
	}
}

fn insert_components_to(agent: &mut EntityCommands, skill: &Skill) {
	agent.insert(TimeTracker::<Skill>::new());
	skill.marker_commands.insert_marker_on(agent);
}

fn remove_components_from(agent: &mut EntityCommands, skill: &Skill) {
	agent.remove::<(Skill, TimeTracker<Skill>)>();
	skill.marker_commands.remove_marker_on(agent);
}

fn update_duration_with<'a>(delta: Duration) -> impl Fn(TrackedSkills<'a>) -> TrackedSkills<'a> {
	move |(agent, skill, mut tracker, wait_next)| {
		tracker.duration += delta;
		(agent, skill, tracker, wait_next)
	}
}

fn skill_is_done((_, skill, tracker, wait_next): &TrackedSkills) -> bool {
	wait_next.is_some() || tracker.duration >= skill.cast.pre + skill.cast.after
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
				marker_commands: Marker::<Tag>::commands(),
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
				marker_commands: Marker::<Tag>::commands(),
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
				marker_commands: Marker::<Tag>::commands(),
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
				marker_commands: Marker::<Tag>::commands(),
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
				marker_commands: Marker::<Tag>::commands(),
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
				marker_commands: Marker::<Tag>::commands(),
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
				marker_commands: Marker::<Tag>::commands(),
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
				marker_commands: Marker::<Tag>::commands(),
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
				marker_commands: Marker::<Tag>::commands(),
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
				marker_commands: Marker::<Tag>::commands(),
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
				marker_commands: Marker::<Tag>::commands(),
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
				marker_commands: Marker::<Tag>::commands(),
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
}
