use crate::{
	behaviors::meta::{Outdated, Target},
	components::{
		PlayerSkills,
		Queue,
		Schedule,
		ScheduleMode,
		SideUnset,
		SlotKey,
		Track,
		WaitNext,
	},
	resources::{CamRay, MouseHover},
	skill::{Active, Queued, Skill},
};
use bevy::{
	ecs::system::{EntityCommands, Res},
	prelude::{Commands, Entity, Query},
	transform::components::GlobalTransform,
};

type Components<'a> = (
	Entity,
	&'a Schedule,
	&'a mut Queue,
	Option<&'a Track<Skill<PlayerSkills<SideUnset>, Active>>>,
);

pub fn enqueue(
	mut agents: Query<Components>,
	mut commands: Commands,
	transforms: Query<&GlobalTransform>,
	cam_ray: Res<CamRay>,
	mouse_hover: Res<MouseHover>,
) {
	if agents.is_empty() {
		return;
	}

	let select = get_select(&cam_ray, &mouse_hover, &transforms);

	for (agent, schedule, mut queue, tracks) in &mut agents {
		enqueue_skills(agent, schedule, &mut queue, tracks, &mut commands, &select);
		commands.entity(agent).remove::<Schedule>();
	}
}

fn get_select(
	ray: &CamRay,
	hover: &MouseHover,
	transforms: &Query<&GlobalTransform>,
) -> Option<Target> {
	let ray = ray.0?;
	let collider = get_outdated(hover.collider, transforms);
	let root = get_outdated(hover.root, transforms);

	Some(Target {
		ray,
		hover: MouseHover { root, collider },
	})
}

fn get_outdated(entity: Option<Entity>, transforms: &Query<&GlobalTransform>) -> Option<Outdated> {
	let entity = entity?;
	let transform = transforms.get(entity).cloned().ok()?;
	Some(Outdated { entity, transform })
}

fn enqueue_skills(
	agent: Entity,
	schedule: &Schedule,
	queue: &mut Queue,
	track: Option<&Track<Skill<PlayerSkills<SideUnset>, Active>>>,
	commands: &mut Commands,
	select: &Option<Target>,
) {
	for scheduled in &schedule.skills {
		enqueue_skill(agent, schedule, queue, track, scheduled, commands, select);
	}
}

fn enqueue_skill(
	agent: Entity,
	schedule: &Schedule,
	queue: &mut Queue,
	track: Option<&Track<Skill<PlayerSkills<SideUnset>, Active>>>,
	(slot, skill): (&SlotKey, &Skill),
	commands: &mut Commands,
	select: &Option<Target>,
) {
	let slot_key = *slot;
	let new = select.clone().map(|select_info| {
		skill.clone().with(&Queued {
			select_info,
			slot_key,
		})
	});
	let Some(new) = new else {
		return;
	};

	match (schedule.mode, track) {
		(ScheduleMode::Override, Some(track)) if track.value.soft_override && new.soft_override => {
			override_soft(queue, &new);
		}
		(ScheduleMode::Override, _) => {
			override_hard(queue, &new, &mut commands.entity(agent));
		}
		(ScheduleMode::Enqueue, _) => {
			enqueue_to(queue, &new);
		}
	}
}

fn enqueue_to(queue: &mut Queue, new: &Skill<PlayerSkills<SideUnset>, Queued>) {
	queue.0.push_back(new.clone());
}

fn override_soft(queue: &mut Queue, new: &Skill<PlayerSkills<SideUnset>, Queued>) {
	queue.0 = vec![new.clone()].into();
}

fn override_hard(
	queue: &mut Queue,
	new: &Skill<PlayerSkills<SideUnset>, Queued>,
	agent: &mut EntityCommands,
) {
	queue.0 = vec![new.clone()].into();
	agent.insert(WaitNext);
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		behaviors::meta::Outdated,
		components::{Schedule, ScheduleMode, Side, WaitNext},
		resources::MouseHover,
		skill::Cast,
	};
	use bevy::{
		prelude::{App, Ray, Update, Vec3},
		transform::components::GlobalTransform,
		utils::default,
	};
	use std::time::Duration;

	const TEST_RAY: Ray = Ray {
		origin: Vec3::ONE,
		direction: Vec3::Z,
	};

	fn setup(ray: Option<Ray>) -> App {
		let mut app = App::new();

		let root = app.world.spawn(GlobalTransform::from_xyz(1., 2., 3.)).id();
		let collider = app.world.spawn(GlobalTransform::from_xyz(4., 5., 6.)).id();

		app.insert_resource(CamRay(ray));
		app.insert_resource(MouseHover {
			collider: Some(collider),
			root: Some(root),
		});
		app.add_systems(Update, enqueue);

		app
	}

	fn root_and_collider_hover(app: &App) -> Option<(Outdated, Outdated)> {
		let hover = app.world.get_resource::<MouseHover<Entity>>()?;
		let root = hover.root?;
		let collider = hover.collider?;
		let root_transform = app.world.entity(root).get::<GlobalTransform>()?;
		let collider_transform = app.world.entity(collider).get::<GlobalTransform>()?;

		Some((
			Outdated {
				entity: root,
				transform: *root_transform,
			},
			Outdated {
				entity: collider,
				transform: *collider_transform,
			},
		))
	}

	fn mouse_hover_target_info(app: &App) -> MouseHover<Outdated> {
		let Some((root, collider)) = root_and_collider_hover(app) else {
			return MouseHover::default();
		};

		MouseHover {
			root: Some(root),
			collider: Some(collider),
		}
	}

	#[test]
	fn set_enqueue() {
		let mut app = setup(Some(TEST_RAY));
		let agent = app
			.world
			.spawn((
				Schedule {
					mode: ScheduleMode::Enqueue,
					skills: [(
						SlotKey::Hand(Side::Off),
						Skill {
							cast: Cast {
								pre: Duration::from_millis(100),
								..default()
							},
							..default()
						},
					)]
					.into(),
				},
				Queue::<PlayerSkills<SideUnset>>(
					[
						Skill {
							cast: Cast {
								pre: Duration::from_millis(1),
								..default()
							},
							..default()
						},
						Skill {
							cast: Cast {
								pre: Duration::from_millis(2),
								..default()
							},
							..default()
						},
					]
					.into(),
				),
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);
		let queue = agent.get::<Queue>().unwrap();

		assert_eq!(
			vec![
				&Skill {
					cast: Cast {
						pre: Duration::from_millis(1),
						..default()
					},
					..default()
				},
				&Skill {
					cast: Cast {
						pre: Duration::from_millis(2),
						..default()
					},
					..default()
				},
				&Skill {
					cast: Cast {
						pre: Duration::from_millis(100),
						..default()
					},
					data: Queued {
						select_info: Target {
							ray: TEST_RAY,
							hover: mouse_hover_target_info(&app),
						},
						slot_key: SlotKey::Hand(Side::Off),
					},
					..default()
				},
			],
			queue
				.0
				.iter()
				.collect::<Vec<&Skill<PlayerSkills<SideUnset>, Queued>>>()
		);
	}

	#[test]
	fn set_override() {
		let mut app = setup(Some(TEST_RAY));
		let new_skill = Skill {
			cast: Cast {
				pre: Duration::from_millis(100),
				..default()
			},
			..default()
		};
		let agent = app
			.world
			.spawn((
				Schedule {
					mode: ScheduleMode::Override,
					skills: [(SlotKey::Hand(Side::Off), new_skill.clone())].into(),
				},
				Queue::<PlayerSkills<SideUnset>>(
					[
						Skill {
							cast: Cast {
								pre: Duration::from_millis(1),
								..default()
							},
							..default()
						},
						Skill {
							cast: Cast {
								pre: Duration::from_millis(2),
								..default()
							},
							..default()
						},
					]
					.into(),
				),
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);
		let queue = agent.get::<Queue>().unwrap();

		assert_eq!(
			(
				vec![&new_skill.with(&Queued {
					select_info: Target {
						ray: TEST_RAY,
						hover: mouse_hover_target_info(&app),
					},
					slot_key: SlotKey::Hand(Side::Off),
				})],
				true
			),
			(queue.0.iter().collect(), agent.contains::<WaitNext>())
		);
	}

	#[test]
	fn set_override_without_wait_next_when_new_and_running_soft_override() {
		let mut app = setup(Some(TEST_RAY));
		let running_skill = Skill {
			name: "running current",
			soft_override: true,
			..default()
		};
		let new_skill = Skill {
			name: "new",
			soft_override: true,
			..default()
		};

		let agent = app
			.world
			.spawn((
				Track::new(running_skill.clone()),
				Schedule {
					mode: ScheduleMode::Override,
					skills: [(SlotKey::Hand(Side::Off), new_skill.clone())].into(),
				},
				Queue::default(),
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);
		let queue = agent.get::<Queue>().unwrap();

		assert_eq!(
			(
				vec![&new_skill.with(&Queued {
					select_info: Target {
						ray: TEST_RAY,
						hover: mouse_hover_target_info(&app),
					},
					slot_key: SlotKey::Hand(Side::Off),
				})],
				&Track::new(running_skill),
				false,
			),
			(
				queue
					.0
					.iter()
					.collect::<Vec<&Skill<PlayerSkills<SideUnset>, Queued>>>(),
				agent
					.get::<Track<Skill<PlayerSkills<SideUnset>, Active>>>()
					.unwrap(),
				agent.contains::<WaitNext>(),
			)
		);
	}

	#[test]
	fn set_override_with_wait_next_when_soft_override_running_soft_override_false() {
		let mut app = setup(Some(TEST_RAY));
		let running_skill = Skill {
			name: "running",
			soft_override: false,
			..default()
		};
		let new_skill = Skill {
			name: "new",
			soft_override: true,
			..default()
		};

		let agent = app
			.world
			.spawn((
				Track::new(running_skill.clone()),
				Schedule {
					mode: ScheduleMode::Override,
					skills: [(SlotKey::Hand(Side::Off), new_skill.clone())].into(),
				},
				Queue::default(),
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);
		let queue = agent.get::<Queue>().unwrap();

		assert_eq!(
			(
				vec![&new_skill.with(&Queued {
					select_info: Target {
						ray: TEST_RAY,
						hover: mouse_hover_target_info(&app),
					},
					slot_key: SlotKey::Hand(Side::Off),
				})],
				&Track::new(running_skill),
				true,
			),
			(
				queue
					.0
					.iter()
					.collect::<Vec<&Skill<PlayerSkills<SideUnset>, Queued>>>(),
				agent
					.get::<Track<Skill<PlayerSkills<SideUnset>, Active>>>()
					.unwrap(),
				agent.contains::<WaitNext>(),
			)
		);
	}

	#[test]
	fn set_override_with_wait_next_when_soft_override_new_soft_override_false() {
		let mut app = setup(Some(TEST_RAY));
		let running_skill = Skill {
			name: "running",
			soft_override: true,
			..default()
		};
		let new_skill = Skill {
			name: "new",
			soft_override: false,
			..default()
		};

		let agent = app
			.world
			.spawn((
				Track::new(running_skill.clone()),
				Schedule {
					mode: ScheduleMode::Override,
					skills: [(SlotKey::Hand(Side::Off), new_skill.clone())].into(),
				},
				Queue::default(),
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);
		let queue = agent.get::<Queue>().unwrap();

		assert_eq!(
			(
				vec![&new_skill.with(&Queued {
					select_info: Target {
						ray: TEST_RAY,
						hover: mouse_hover_target_info(&app),
					},
					slot_key: SlotKey::Hand(Side::Off),
				})],
				&Track::new(running_skill),
				true,
			),
			(
				queue
					.0
					.iter()
					.collect::<Vec<&Skill<PlayerSkills<SideUnset>, Queued>>>(),
				agent
					.get::<Track<Skill<PlayerSkills<SideUnset>, Active>>>()
					.unwrap(),
				agent.contains::<WaitNext>(),
			)
		);
	}

	#[test]
	fn remove_schedule() {
		let mut app = setup(Some(TEST_RAY));
		let schedule = Schedule {
			mode: ScheduleMode::Override,
			skills: [(SlotKey::Hand(Side::Off), Skill::default())].into(),
		};
		let agent = app.world.spawn((schedule, Queue::default())).id();

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Schedule>());
	}

	#[test]
	fn try_soft_override_on_enqueue() {
		let mut app = setup(Some(TEST_RAY));
		app.world.spawn((
			Schedule {
				mode: ScheduleMode::Enqueue,
				skills: [(SlotKey::Hand(Side::Off), Skill::default())].into(),
			},
			Track::new(Skill::<PlayerSkills<SideUnset>, Active>::default()),
			Queue::default(),
		));

		app.update();
	}
}
