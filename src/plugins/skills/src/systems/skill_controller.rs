use super::get_inputs::Input;
use crate::{
	components::{SlotKey, Slots, Track},
	skill::{Active, Queued, Skill},
	traits::{Enqueue, IterMut},
};
use bevy::{
	ecs::{
		component::Component,
		system::{In, Local, Query, Res},
	},
	time::Time,
};
use std::{collections::HashMap, time::Duration};

type TrackTime = HashMap<SlotKey, Duration>;
type Components<'a, TQueue> = (
	&'a Slots,
	&'a mut TQueue,
	Option<&'a mut Track<Skill<Active>>>,
);

pub(crate) fn skill_controller<
	TQueue: Component + Enqueue<Skill<Queued>> + IterMut<Skill<Queued>>,
	TTime: Default + Send + Sync + 'static,
>(
	input: In<Input>,
	time: Res<Time<TTime>>,
	mut agents: Query<Components<TQueue>>,
	mut times: Local<TrackTime>,
) {
	for (slots, mut queue, mut active) in &mut agents {
		let queue = queue.as_mut();
		enqueue_new_skills(&input, &mut times, &time, queue, slots);
		update_skill_aim_times(&input, &times, &time, queue, active.as_deref_mut());
	}
}

fn enqueue_new_skills<TQueue: Enqueue<Skill<Queued>>, TTime: Default + Send + Sync + 'static>(
	input: &In<Input>,
	times: &mut Local<TrackTime>,
	time: &Res<Time<TTime>>,
	queue: &mut TQueue,
	slots: &Slots,
) {
	for key in input.just_pressed.iter() {
		enqueue_new_skill(key, slots, queue, times, time);
	}
}

fn enqueue_new_skill<TQueue: Enqueue<Skill<Queued>>, TTime: Default + Send + Sync + 'static>(
	key: &SlotKey,
	slots: &Slots,
	queue: &mut TQueue,
	times: &mut Local<TrackTime>,
	time: &Res<Time<TTime>>,
) {
	let Some(skill) = get_slot_skill(key, slots) else {
		return;
	};
	times.insert(*key, time.elapsed());
	queue.enqueue(skill.with(Queued(*key)));
}

fn get_slot_skill(key: &SlotKey, slots: &Slots) -> Option<Skill> {
	slots
		.0
		.get(key)
		.and_then(|s| s.item.clone())
		.and_then(|i| i.skill)
}

fn update_skill_aim_times<
	TQueue: IterMut<Skill<Queued>>,
	TTime: Default + Send + Sync + 'static,
>(
	input: &In<Input>,
	times: &Local<HashMap<SlotKey, Duration>>,
	time: &Res<Time<TTime>>,
	queue: &mut TQueue,
	mut active_skill: Option<&mut Track<Skill<Active>>>,
) {
	let get_key_time = |key| Some((key, times.get(key)?));

	for (key, duration) in input.just_released.iter().filter_map(get_key_time) {
		update_skill_aim_time(key, time, duration, queue, &mut active_skill);
	}
}

fn update_skill_aim_time<TQueue: IterMut<Skill<Queued>>, TTime: Default + Send + Sync + 'static>(
	key: &SlotKey,
	time: &Res<Time<TTime>>,
	duration: &Duration,
	queue: &mut TQueue,
	active_skill: &mut Option<&mut Track<Skill<Active>>>,
) {
	if update_aim_time_in_queue(key, time, duration, queue) {
		return;
	}
	update_aim_time_on_active(key, time, duration, active_skill);
}

fn update_aim_time_in_queue<
	TQueue: IterMut<Skill<Queued>>,
	TTime: Default + Send + Sync + 'static,
>(
	key: &SlotKey,
	time: &Res<Time<TTime>>,
	duration: &Duration,
	queue: &mut TQueue,
) -> bool {
	let Some(skill) = get_queued_skill(key, queue) else {
		return false;
	};
	skill.cast.aim = time.elapsed() - *duration;
	true
}

fn update_aim_time_on_active<TTime: Default + Send + Sync + 'static>(
	key: &SlotKey,
	time: &Res<Time<TTime>>,
	duration: &Duration,
	active_skill: &mut Option<&mut Track<Skill<Active>>>,
) {
	let Some(skill) = active_skill.as_mut() else {
		return;
	};
	if &skill.value.data.0 != key {
		return;
	}
	skill.value.cast.aim = time.elapsed() - *duration;
}

fn get_queued_skill<'a, TQueue: IterMut<Skill<Queued>>>(
	key: &SlotKey,
	queue: &'a mut TQueue,
) -> Option<&'a mut Skill<Queued>> {
	queue.iter_mut().rev().find(|skill| &skill.data.0 == key)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{Item, Slot, SlotKey, Slots, Track},
		skill::{Active, Cast, Queued, Skill},
	};
	use bevy::{
		app::{App, Update},
		ecs::{
			entity::Entity,
			system::{IntoSystem, Resource},
		},
		time::{Real, Time},
		utils::default,
	};
	use common::{
		components::Side,
		test_tools::utils::{SingleThreadedApp, TickTime},
	};
	use std::time::Duration;

	#[derive(Resource, Default)]
	struct _Input(Input);

	#[derive(Component, Default, Debug, PartialEq)]
	struct _Queue {
		queued: Vec<Skill<Queued>>,
	}

	impl Enqueue<Skill<Queued>> for _Queue {
		fn enqueue(&mut self, item: Skill<Queued>) {
			self.queued.push(item)
		}
	}

	impl IterMut<Skill<Queued>> for _Queue {
		fn iter_mut<'a>(&'a mut self) -> impl DoubleEndedIterator<Item = &'a mut Skill<Queued>>
		where
			Skill<Queued>: 'a,
		{
			self.queued.iter_mut()
		}
	}

	fn setup() -> App {
		let mut app = App::new_single_threaded([Update]);
		app.init_resource::<Time<Real>>();
		app.init_resource::<_Input>();
		app.tick_time(Duration::from_millis(42));
		app.add_systems(
			Update,
			(move |input: Res<_Input>| input.0.clone()).pipe(skill_controller::<_Queue, Real>),
		);

		app
	}

	#[test]
	fn enqueue_skill() {
		let mut app = setup();
		let agent = app
			.world
			.spawn((
				Slots(HashMap::from([(
					SlotKey::Hand(Side::Main),
					Slot {
						entity: Entity::from_raw(42),
						item: Some(Item {
							skill: Some(Skill {
								name: "my skill",
								..default()
							}),
							..default()
						}),
						combo_skill: None,
					},
				)])),
				_Queue::default(),
			))
			.id();

		app.world.resource_mut::<_Input>().0.just_pressed = vec![SlotKey::Hand(Side::Main)];
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&_Queue {
				queued: vec![Skill {
					name: "my skill",
					data: Queued(SlotKey::Hand(Side::Main)),
					..default()
				}]
			}),
			agent.get::<_Queue>()
		);
	}

	#[test]
	fn update_aim_time_for_first_scheduled() {
		let mut app = setup();
		let agent = app
			.world
			.spawn((
				Slots(HashMap::from([(
					SlotKey::Hand(Side::Main),
					Slot {
						entity: Entity::from_raw(42),
						item: Some(Item {
							skill: Some(Skill {
								name: "main",
								..default()
							}),
							..default()
						}),
						combo_skill: None,
					},
				)])),
				_Queue::default(),
			))
			.id();

		app.world.resource_mut::<_Input>().0.just_pressed = vec![SlotKey::Hand(Side::Main)];
		app.update();

		app.tick_time(Duration::from_millis(100));
		app.world.resource_mut::<_Input>().0.just_pressed = vec![];
		app.world.resource_mut::<_Input>().0.just_released = vec![SlotKey::Hand(Side::Main)];
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&_Queue {
				queued: vec![Skill {
					name: "main",
					data: Queued(SlotKey::Hand(Side::Main)),
					cast: Cast {
						aim: Duration::from_millis(100),
						..default()
					},
					..default()
				}]
			}),
			agent.get::<_Queue>(),
		);
	}

	#[test]
	fn update_aim_time_for_first_scheduled_over_multiple_frames() {
		let mut app = setup();
		let agent = app
			.world
			.spawn((
				Slots(HashMap::from([(
					SlotKey::Hand(Side::Main),
					Slot {
						entity: Entity::from_raw(42),
						item: Some(Item {
							skill: Some(Skill {
								name: "main",
								..default()
							}),
							..default()
						}),
						combo_skill: None,
					},
				)])),
				_Queue::default(),
			))
			.id();

		app.world.resource_mut::<_Input>().0.just_pressed = vec![SlotKey::Hand(Side::Main)];
		app.update();

		app.tick_time(Duration::from_millis(100));
		app.world.resource_mut::<_Input>().0.just_pressed = vec![];
		app.update();

		app.tick_time(Duration::from_millis(100));
		app.update();

		app.tick_time(Duration::from_millis(100));
		app.world.resource_mut::<_Input>().0.just_released = vec![SlotKey::Hand(Side::Main)];
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&_Queue {
				queued: vec![Skill {
					name: "main",
					data: Queued(SlotKey::Hand(Side::Main)),
					cast: Cast {
						aim: Duration::from_millis(300),
						..default()
					},
					..default()
				}]
			}),
			agent.get::<_Queue>()
		);
	}

	#[test]
	fn update_aim_time_on_skill_matching_with_key() {
		let mut app = setup();
		let agent = app
			.world
			.spawn((
				Slots(HashMap::from([(
					SlotKey::Hand(Side::Main),
					Slot {
						entity: Entity::from_raw(42),
						item: Some(Item {
							skill: Some(Skill {
								name: "main",
								..default()
							}),
							..default()
						}),
						combo_skill: None,
					},
				)])),
				_Queue {
					queued: vec![Skill {
						data: Queued(SlotKey::Hand(Side::Off)),
						..default()
					}],
				},
			))
			.id();

		app.world.resource_mut::<_Input>().0.just_pressed = vec![SlotKey::Hand(Side::Main)];
		app.update();

		app.tick_time(Duration::from_millis(100));
		app.world.resource_mut::<_Input>().0.just_pressed = vec![];
		app.world.resource_mut::<_Input>().0.just_released = vec![SlotKey::Hand(Side::Main)];
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&_Queue {
				queued: vec![
					Skill {
						data: Queued(SlotKey::Hand(Side::Off)),
						..default()
					},
					Skill {
						name: "main",
						data: Queued(SlotKey::Hand(Side::Main)),
						cast: Cast {
							aim: Duration::from_millis(100),
							..default()
						},
						..default()
					}
				]
			}),
			agent.get::<_Queue>()
		);
	}

	#[test]
	fn update_aim_time_on_skill_depending_on_queue_time() {
		let mut app = setup();
		let agent = app
			.world
			.spawn((
				Slots(HashMap::from([
					(
						SlotKey::Hand(Side::Main),
						Slot {
							entity: Entity::from_raw(42),
							item: Some(Item {
								skill: Some(Skill {
									name: "main",
									..default()
								}),
								..default()
							}),
							combo_skill: None,
						},
					),
					(
						SlotKey::Hand(Side::Off),
						Slot {
							entity: Entity::from_raw(42),
							item: Some(Item {
								skill: Some(Skill {
									name: "off",
									..default()
								}),
								..default()
							}),
							combo_skill: None,
						},
					),
				])),
				_Queue::default(),
			))
			.id();

		app.world.resource_mut::<_Input>().0.just_pressed = vec![SlotKey::Hand(Side::Main)];
		app.update();

		app.tick_time(Duration::from_millis(100));
		app.world.resource_mut::<_Input>().0.just_pressed = vec![SlotKey::Hand(Side::Off)];
		app.update();

		app.tick_time(Duration::from_millis(100));
		app.world.resource_mut::<_Input>().0.just_pressed = vec![];
		app.world.resource_mut::<_Input>().0.just_released =
			vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)];
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&_Queue {
				queued: vec![
					Skill {
						name: "main",
						data: Queued(SlotKey::Hand(Side::Main)),
						cast: Cast {
							aim: Duration::from_millis(200),
							..default()
						},
						..default()
					},
					Skill {
						name: "off",
						data: Queued(SlotKey::Hand(Side::Off)),
						cast: Cast {
							aim: Duration::from_millis(100),
							..default()
						},
						..default()
					},
				]
			}),
			agent.get::<_Queue>()
		);
	}

	#[test]
	fn update_aim_time_from_queue_back() {
		let mut app = setup();
		let agent = app
			.world
			.spawn((
				Slots(HashMap::from([(
					SlotKey::Hand(Side::Main),
					Slot {
						entity: Entity::from_raw(42),
						item: Some(Item {
							skill: Some(Skill {
								name: "main",
								..default()
							}),
							..default()
						}),
						combo_skill: None,
					},
				)])),
				_Queue {
					queued: vec![Skill {
						name: "other",
						data: Queued(SlotKey::Hand(Side::Main)),
						..default()
					}],
				},
			))
			.id();

		app.world.resource_mut::<_Input>().0.just_pressed = vec![SlotKey::Hand(Side::Main)];
		app.update();

		app.tick_time(Duration::from_millis(100));
		app.world.resource_mut::<_Input>().0.just_pressed = vec![];
		app.world.resource_mut::<_Input>().0.just_released = vec![SlotKey::Hand(Side::Main)];
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&_Queue {
				queued: vec![
					Skill {
						name: "other",
						data: Queued(SlotKey::Hand(Side::Main)),
						..default()
					},
					Skill {
						name: "main",
						data: Queued(SlotKey::Hand(Side::Main)),
						cast: Cast {
							aim: Duration::from_millis(100),
							..default()
						},
						..default()
					},
				]
			}),
			agent.get::<_Queue>()
		);
	}

	#[test]
	fn update_aim_time_on_active() {
		let mut app = setup();
		let agent = app
			.world
			.spawn((
				Slots(HashMap::from([(
					SlotKey::Hand(Side::Main),
					Slot {
						entity: Entity::from_raw(42),
						item: Some(Item {
							skill: Some(Skill {
								name: "main",
								..default()
							}),
							..default()
						}),
						combo_skill: None,
					},
				)])),
				_Queue::default(),
			))
			.id();

		app.world.resource_mut::<_Input>().0.just_pressed = vec![SlotKey::Hand(Side::Main)];
		app.update();

		app.world
			.entity_mut(agent)
			.get_mut::<_Queue>()
			.unwrap()
			.queued = vec![];
		app.world.entity_mut(agent).insert(Track::new(Skill {
			data: Active(SlotKey::Hand(Side::Main)),
			..default()
		}));

		app.tick_time(Duration::from_millis(100));
		app.world.resource_mut::<_Input>().0.just_pressed = vec![];
		app.world.resource_mut::<_Input>().0.just_released = vec![SlotKey::Hand(Side::Main)];
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&Track::new(Skill {
				data: Active(SlotKey::Hand(Side::Main)),
				cast: Cast {
					aim: Duration::from_millis(100),
					..default()
				},
				..default()
			},)),
			agent.get::<Track<Skill<Active>>>()
		);
	}

	#[test]
	fn do_not_update_aim_time_on_active_if_slot_key_does_not_match() {
		let mut app = setup();
		let agent = app
			.world
			.spawn((
				Slots(HashMap::from([(
					SlotKey::Hand(Side::Main),
					Slot {
						entity: Entity::from_raw(42),
						item: Some(Item {
							skill: Some(Skill {
								name: "main",
								..default()
							}),
							..default()
						}),
						combo_skill: None,
					},
				)])),
				_Queue::default(),
			))
			.id();

		app.world.resource_mut::<_Input>().0.just_pressed = vec![SlotKey::Hand(Side::Main)];
		app.update();

		app.world
			.entity_mut(agent)
			.get_mut::<_Queue>()
			.unwrap()
			.queued = vec![];
		app.world.entity_mut(agent).insert(Track::new(Skill {
			data: Active(SlotKey::Hand(Side::Off)),
			..default()
		}));

		app.tick_time(Duration::from_millis(100));
		app.world.resource_mut::<_Input>().0.just_pressed = vec![];
		app.world.resource_mut::<_Input>().0.just_released = vec![SlotKey::Hand(Side::Main)];
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&Track::new(Skill {
				data: Active(SlotKey::Hand(Side::Off)),
				..default()
			},)),
			agent.get::<Track<Skill<Active>>>()
		);
	}

	#[test]
	fn update_aim_time_on_active_over_multiple_frames() {
		let mut app = setup();
		let agent = app
			.world
			.spawn((
				Slots(HashMap::from([(
					SlotKey::Hand(Side::Main),
					Slot {
						entity: Entity::from_raw(42),
						item: Some(Item {
							skill: Some(Skill {
								name: "main",
								..default()
							}),
							..default()
						}),
						combo_skill: None,
					},
				)])),
				_Queue::default(),
			))
			.id();

		app.world.resource_mut::<_Input>().0.just_pressed = vec![SlotKey::Hand(Side::Main)];
		app.update();

		app.world
			.entity_mut(agent)
			.get_mut::<_Queue>()
			.unwrap()
			.queued = vec![];
		app.world.entity_mut(agent).insert(Track::new(Skill {
			data: Active(SlotKey::Hand(Side::Main)),
			..default()
		}));

		app.tick_time(Duration::from_millis(100));
		app.world.resource_mut::<_Input>().0.just_pressed = vec![];
		app.update();

		app.tick_time(Duration::from_millis(100));
		app.world.resource_mut::<_Input>().0.just_released = vec![SlotKey::Hand(Side::Main)];
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&Track::new(Skill {
				data: Active(SlotKey::Hand(Side::Main)),
				cast: Cast {
					aim: Duration::from_millis(200),
					..default()
				},
				..default()
			},)),
			agent.get::<Track<Skill<Active>>>()
		);
	}

	#[test]
	fn update_aim_time_in_queue_even_if_same_key_is_active() {
		let mut app = setup();
		let agent = app
			.world
			.spawn((
				Slots(HashMap::from([(
					SlotKey::Hand(Side::Main),
					Slot {
						entity: Entity::from_raw(42),
						item: Some(Item {
							skill: Some(Skill {
								name: "main",
								..default()
							}),
							..default()
						}),
						combo_skill: None,
					},
				)])),
				_Queue::default(),
			))
			.id();

		app.world.resource_mut::<_Input>().0.just_pressed = vec![SlotKey::Hand(Side::Main)];
		app.update();

		app.world.entity_mut(agent).insert(Track::new(Skill {
			data: Active(SlotKey::Hand(Side::Main)),
			..default()
		}));

		app.tick_time(Duration::from_millis(100));
		app.world.resource_mut::<_Input>().0.just_pressed = vec![];
		app.world.resource_mut::<_Input>().0.just_released = vec![SlotKey::Hand(Side::Main)];
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			(
				Some(&_Queue {
					queued: vec![Skill {
						name: "main",
						data: Queued(SlotKey::Hand(Side::Main)),
						cast: Cast {
							aim: Duration::from_millis(100),
							..default()
						},
						..default()
					},]
				}),
				Some(&Track::new(Skill {
					data: Active(SlotKey::Hand(Side::Main)),
					..default()
				},))
			),
			(agent.get::<_Queue>(), agent.get::<Track<Skill<Active>>>())
		);
	}
}
