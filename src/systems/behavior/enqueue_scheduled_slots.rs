use bevy::{
	prelude::{Camera, Commands, Component, Entity, GlobalTransform, Query, Ray, With},
	window::Window,
};

use crate::{
	components::{Idle, Queue, Schedule, Slot, Slots},
	traits::get_ray::GetRayFromCamera,
};

type Components<'a, TBehavior> = (Entity, &'a mut Slots<TBehavior>, &'a mut Queue<TBehavior>);

fn enqueue_slot<TBehavior>(
	agent: Entity,
	slot: &mut Slot<TBehavior>,
	queue: &mut Queue<TBehavior>,
	commands: &mut Commands,
	ray: Option<Ray>,
) {
	let (Some(get_behavior), Some(ray)) = (slot.get_behavior, ray) else {
		return;
	};

	let Some(behavior) = get_behavior(ray) else {
		return;
	};

	match slot.schedule {
		Some(Schedule::Enqueue) => queue.0.push_back(behavior),
		Some(Schedule::Override) => {
			queue.0.clear();
			queue.0.push_back(behavior);
			commands.entity(agent).insert(Idle);
		}
		None => {}
	}
	slot.schedule = None;
}

fn enqueue_slots<TBehavior>(
	agent: Entity,
	slots: &mut Slots<TBehavior>,
	queue: &mut Queue<TBehavior>,
	commands: &mut Commands,
	ray: Option<Ray>,
) {
	for slot in slots.0.values_mut() {
		enqueue_slot(agent, slot, queue, commands, ray);
	}
}

pub fn enqueue_scheduled_slots<
	TAgent: Component,
	TBehavior: Sync + Send + 'static,
	TGetRay: GetRayFromCamera,
>(
	camera: Query<(&Camera, &GlobalTransform)>,
	window: Query<&Window>,
	mut agents: Query<Components<TBehavior>, With<TAgent>>,
	mut commands: Commands,
) {
	let (camera, camera_transform) = camera.single();
	let window = window.single();
	let ray = TGetRay::get_ray(camera, camera_transform, window);

	for (agent, mut slots, mut queue) in &mut agents {
		enqueue_slots(agent, &mut slots, &mut queue, &mut commands, ray);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{Idle, Schedule, Slot, SlotKey};
	use bevy::{
		prelude::{App, Camera, Camera3dBundle, GlobalTransform, Ray, Update, Vec3},
		utils::default,
		window::Window,
	};
	use mockall::automock;

	#[derive(Component)]
	struct Agent;

	#[derive(PartialEq, Debug)]
	struct MockBehavior {
		pub ray: Ray,
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
			enqueue_scheduled_slots::<Agent, MockBehavior, TGetRay>,
		);

		app
	}

	#[test]
	fn enqueue() {
		let mut app = setup::<GetDefaultRay>();
		let agent = app
			.world
			.spawn((
				Agent,
				Slots(
					[(
						SlotKey::Legs,
						Slot {
							entity: Entity::from_raw(0),
							schedule: Some(Schedule::Enqueue),
							get_behavior: Some(|ray| Some(MockBehavior { ray })),
						},
					)]
					.into(),
				),
				Queue(
					vec![
						MockBehavior {
							ray: Ray {
								origin: Vec3::Z,
								direction: Vec3::Y,
							},
						},
						MockBehavior {
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
					ray: Ray {
						origin: Vec3::Z,
						direction: Vec3::Y,
					},
				},
				&MockBehavior {
					ray: Ray {
						origin: Vec3::Z,
						direction: Vec3::Y,
					},
				},
				&MockBehavior { ray: DEFAULT_RAY }
			],
			queue.0.iter().collect::<Vec<&MockBehavior>>()
		);
	}

	#[test]
	fn override_queue() {
		let mut app = setup::<GetDefaultRay>();
		let agent = app
			.world
			.spawn((
				Agent,
				Slots(
					[(
						SlotKey::Legs,
						Slot {
							entity: Entity::from_raw(0),
							schedule: Some(Schedule::Override),
							get_behavior: Some(|ray| Some(MockBehavior { ray })),
						},
					)]
					.into(),
				),
				Queue(
					vec![
						MockBehavior {
							ray: Ray {
								origin: Vec3::Z,
								direction: Vec3::Y,
							},
						},
						MockBehavior {
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
			(vec![&MockBehavior { ray: DEFAULT_RAY }], true),
			(
				queue.0.iter().collect::<Vec<&MockBehavior>>(),
				agent.contains::<Idle>()
			)
		);
	}

	#[test]
	fn reset_schedule() {
		let mut app = setup::<GetDefaultRay>();
		let agent = app
			.world
			.spawn((
				Agent,
				Slots(
					[(
						SlotKey::Legs,
						Slot {
							entity: Entity::from_raw(0),
							schedule: Some(Schedule::Enqueue),
							get_behavior: Some(|ray| Some(MockBehavior { ray })),
						},
					)]
					.into(),
				),
				Queue::<MockBehavior>::new(),
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);
		let slots = agent.get::<Slots<MockBehavior>>().unwrap();
		let slot = slots.0.values().next().unwrap();

		assert_eq!(None, slot.schedule);
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
			Agent,
			Slots(
				[(
					SlotKey::Legs,
					Slot {
						entity: Entity::from_raw(0),
						schedule: Some(Schedule::Enqueue),
						get_behavior: Some(|ray| Some(MockBehavior { ray })),
					},
				)]
				.into(),
			),
			Queue::<MockBehavior>::new(),
		));

		app.update();
	}
}
