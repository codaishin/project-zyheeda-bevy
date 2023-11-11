use crate::{
	components::{Queue, Schedule, ScheduleMode, SlotKey, WaitNext},
	traits::{from_source::FromSource, get_ray::GetRayFromCamera},
};
use bevy::{
	prelude::{Camera, Commands, Entity, GlobalTransform, Query, Ray},
	window::Window,
};

fn enqueue_agent_behavior<
	TBehaviorItem: Copy,
	TBehavior: FromSource<TBehaviorItem, (SlotKey, Ray)> + Copy + Send + Sync + 'static,
>(
	agent: Entity,
	queue: &mut Queue<TBehavior>,
	schedule_mode: ScheduleMode,
	behavior_slot: (&SlotKey, &TBehaviorItem),
	commands: &mut Commands,
	ray: Option<Ray>,
) {
	let (slot_key, behavior) = behavior_slot;
	let complete_behavior = |ray| TBehavior::from_source(*behavior, (*slot_key, ray));

	let Some(behavior) = ray.and_then(complete_behavior) else {
		return;
	};

	match schedule_mode {
		ScheduleMode::Enqueue => queue.0.push_back(behavior),
		ScheduleMode::Override => {
			queue.0.clear();
			queue.0.push_back(behavior);
			commands.entity(agent).insert(WaitNext::<TBehavior>::new());
		}
	}
}

fn enqueue_agent_behaviors<
	TBehaviorItem: Copy,
	TBehavior: FromSource<TBehaviorItem, (SlotKey, Ray)> + Copy + Send + Sync + 'static,
>(
	agent: Entity,
	schedule: &Schedule<TBehaviorItem>,
	queue: &mut Queue<TBehavior>,
	commands: &mut Commands,
	ray: Option<Ray>,
) {
	for behavior_slot in &schedule.behaviors {
		enqueue_agent_behavior(agent, queue, schedule.mode, behavior_slot, commands, ray);
	}
}

type Components<'a, TBehaviorItem, TBehavior> = (
	Entity,
	&'a Schedule<TBehaviorItem>,
	&'a mut Queue<TBehavior>,
);

pub fn enqueue<
	TBehaviorItem: Copy + Send + Sync + 'static,
	TBehavior: FromSource<TBehaviorItem, (SlotKey, Ray)> + Copy + Send + Sync + 'static,
	TGetRay: GetRayFromCamera,
>(
	camera: Query<(&Camera, &GlobalTransform)>,
	window: Query<&Window>,
	mut agents: Query<Components<TBehaviorItem, TBehavior>>,
	mut commands: Commands,
) {
	if agents.is_empty() {
		return;
	}

	let (camera, camera_transform) = camera.single();
	let window = window.single();
	let ray = TGetRay::get_ray(camera, camera_transform, window);

	for (agent, schedule, mut queue) in &mut agents {
		enqueue_agent_behaviors(agent, schedule, &mut queue, &mut commands, ray);
		commands.entity(agent).remove::<Schedule<TBehaviorItem>>();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{Schedule, ScheduleMode, Side, WaitNext};
	use bevy::{
		prelude::{App, Camera, Camera3dBundle, GlobalTransform, Ray, Update, Vec3},
		utils::default,
		window::Window,
	};
	use mockall::automock;

	#[derive(PartialEq, Debug, Clone, Copy)]
	struct MockBehaviorPartial {
		pub id: u32,
	}

	#[derive(PartialEq, Debug, Clone, Copy)]
	struct MockBehavior {
		pub id: u32,
		pub slot_key: SlotKey,
		pub ray: Ray,
	}

	impl FromSource<MockBehaviorPartial, (SlotKey, Ray)> for MockBehavior {
		fn from_source(
			source: MockBehaviorPartial,
			(slot_key, ray): (SlotKey, Ray),
		) -> Option<Self> {
			Some(Self {
				id: source.id,
				slot_key,
				ray,
			})
		}
	}

	const DEFAULT_RAY: Ray = Ray {
		origin: Vec3::ONE,
		direction: Vec3::Z,
	};

	struct GetDefaultRay;

	impl GetRayFromCamera for GetDefaultRay {
		fn get_ray(
			_camera: &Camera,
			_camera_transform: &GlobalTransform,
			_window: &Window,
		) -> Option<Ray> {
			Some(DEFAULT_RAY)
		}
	}

	fn setup<TGetRay: GetRayFromCamera + 'static>() -> App {
		let mut app = App::new();

		app.world.spawn(Camera3dBundle {
			camera: Camera {
				order: 42,
				..default()
			},
			global_transform: GlobalTransform::from_xyz(4., 3., 2.),
			..default()
		});
		app.world.spawn(Window {
			title: "Window".to_owned(),
			..default()
		});
		app.add_systems(
			Update,
			enqueue::<MockBehaviorPartial, MockBehavior, TGetRay>,
		);

		app
	}

	#[test]
	fn set_enqueue() {
		let mut app = setup::<GetDefaultRay>();
		let agent = app
			.world
			.spawn((
				Schedule {
					mode: ScheduleMode::Enqueue,
					behaviors: [(SlotKey::Hand(Side::Left), MockBehaviorPartial { id: 42 })].into(),
				},
				Queue(
					[
						MockBehavior {
							id: 1,
							slot_key: SlotKey::Legs,
							ray: Ray {
								origin: Vec3::Z,
								direction: Vec3::Y,
							},
						},
						MockBehavior {
							id: 2,
							slot_key: SlotKey::Hand(Side::Right),
							ray: Ray {
								origin: Vec3::Z,
								direction: Vec3::Y,
							},
						},
					]
					.into(),
				),
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);
		let queue = agent.get::<Queue<MockBehavior>>().unwrap();

		assert_eq!(
			vec![
				&MockBehavior {
					id: 1,
					slot_key: SlotKey::Legs,
					ray: Ray {
						origin: Vec3::Z,
						direction: Vec3::Y,
					},
				},
				&MockBehavior {
					id: 2,
					slot_key: SlotKey::Hand(Side::Right),
					ray: Ray {
						origin: Vec3::Z,
						direction: Vec3::Y,
					},
				},
				&MockBehavior {
					id: 42,
					slot_key: SlotKey::Hand(Side::Left),
					ray: DEFAULT_RAY
				}
			],
			queue.0.iter().collect::<Vec<&MockBehavior>>()
		);
	}

	#[test]
	fn set_override() {
		let mut app = setup::<GetDefaultRay>();
		let agent = app
			.world
			.spawn((
				Schedule {
					mode: ScheduleMode::Override,
					behaviors: [(SlotKey::Hand(Side::Left), MockBehaviorPartial { id: 42 })].into(),
				},
				Queue(
					[
						MockBehavior {
							id: 1,
							slot_key: SlotKey::Legs,
							ray: Ray {
								origin: Vec3::Z,
								direction: Vec3::Y,
							},
						},
						MockBehavior {
							id: 2,
							slot_key: SlotKey::Hand(Side::Right),
							ray: Ray {
								origin: Vec3::Z,
								direction: Vec3::Y,
							},
						},
					]
					.into(),
				),
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);
		let queue = agent.get::<Queue<MockBehavior>>().unwrap();

		assert_eq!(
			(
				vec![&MockBehavior {
					id: 42,
					slot_key: SlotKey::Hand(Side::Left),
					ray: DEFAULT_RAY
				}],
				true
			),
			(
				queue.0.iter().collect(),
				agent.contains::<WaitNext<MockBehavior>>()
			)
		);
	}

	#[test]
	fn remove_schedule() {
		let mut app = setup::<GetDefaultRay>();
		let agent = app
			.world
			.spawn((
				Schedule {
					mode: ScheduleMode::Override,
					behaviors: [(SlotKey::Hand(Side::Left), MockBehaviorPartial { id: 42 })].into(),
				},
				Queue::<MockBehavior>([].into()),
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Schedule<MockBehaviorPartial>>());
	}

	#[test]
	fn ray_from_camera_and_window() {
		struct _GetRay;

		#[automock]
		impl GetRayFromCamera for _GetRay {
			fn get_ray(
				_camera: &Camera,
				_camera_transform: &GlobalTransform,
				_window: &Window,
			) -> Option<Ray> {
				None
			}
		}

		let mut app = setup::<Mock_GetRay>();
		let get_ray = Mock_GetRay::get_ray_context();
		let ray = Ray {
			origin: Vec3::ZERO,
			direction: Vec3::ONE,
		};

		get_ray
			.expect()
			.withf(|cam, cam_transform, window| {
				*cam_transform == GlobalTransform::from_xyz(4., 3., 2.)
				// using specific values for non-equatable variables
				&& cam.order == 42 && window.title == "Window"
			})
			.times(1)
			.return_const(ray);

		app.world.spawn((
			Schedule {
				mode: ScheduleMode::Override,
				behaviors: [(SlotKey::Hand(Side::Left), MockBehaviorPartial { id: 42 })].into(),
			},
			Queue::<MockBehavior>([].into()),
		));

		app.update();
	}

	#[test]
	fn do_not_produce_ray_when_nothing_scheduled() {
		struct _GetRay;

		#[automock]
		impl GetRayFromCamera for _GetRay {
			fn get_ray(
				_camera: &Camera,
				_camera_transform: &GlobalTransform,
				_window: &Window,
			) -> Option<Ray> {
				None
			}
		}

		let mut app = setup::<Mock_GetRay>();
		let get_ray = Mock_GetRay::get_ray_context();
		let ray = Ray {
			origin: Vec3::ZERO,
			direction: Vec3::ONE,
		};

		get_ray.expect().times(0).return_const(ray);

		app.world.spawn(Queue::<MockBehavior>([].into()));

		app.update();
	}
}
