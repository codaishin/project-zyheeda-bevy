use super::get_input::Input;
use crate::{
	components::{Queue, SideUnset, SlotKey, Slots, Track},
	skill::{Active, PlayerSkills, Queued, Skill},
};
use bevy::{
	ecs::system::{In, Local, Query, Res},
	time::Time,
};
use std::{collections::HashMap, time::Duration};

type TrackTime = HashMap<SlotKey, Duration>;
type Components<'a> = (
	&'a Slots,
	&'a mut Queue,
	Option<&'a mut Track<Skill<PlayerSkills<SideUnset>, Active>>>,
);

pub(crate) fn skill_controller<TTime: Default + Send + Sync + 'static>(
	input: In<Input>,
	time: Res<Time<TTime>>,
	mut agents: Query<Components>,
	mut times: Local<TrackTime>,
) {
	for (slots, mut queue, mut active) in &mut agents {
		enqueue_new_skills(&input, &mut times, &time, &mut queue, slots);
		update_skill_aim_times(&input, &times, &time, &mut queue, active.as_deref_mut());
	}
}

fn enqueue_new_skills<TTime: Default + Send + Sync + 'static>(
	input: &In<Input>,
	times: &mut Local<TrackTime>,
	time: &Res<Time<TTime>>,
	queue: &mut Queue,
	slots: &Slots,
) {
	for key in input.just_pressed.iter() {
		enqueue_new_skill(key, slots, queue, times, time);
	}
}

fn enqueue_new_skill<TTime: Default + Send + Sync + 'static>(
	key: &SlotKey,
	slots: &Slots,
	queue: &mut Queue,
	times: &mut Local<TrackTime>,
	time: &Res<Time<TTime>>,
) {
	let Some(skill) = get_slot_skill(key, slots) else {
		return;
	};
	times.insert(*key, time.elapsed());
	queue.0.push_back(skill.with(Queued(*key)));
}

fn get_slot_skill(key: &SlotKey, slots: &Slots) -> Option<Skill> {
	slots
		.0
		.get(key)
		.and_then(|s| s.item.clone())
		.and_then(|i| i.skill)
}

fn update_skill_aim_times<TTime: Default + Send + Sync + 'static>(
	input: &In<Input>,
	times: &Local<HashMap<SlotKey, Duration>>,
	time: &Res<Time<TTime>>,
	queue: &mut Queue,
	mut active_skill: Option<&mut Track<Skill<PlayerSkills<SideUnset>, Active>>>,
) {
	let get_key_time = |key| Some((key, times.get(key)?));

	for (key, duration) in input.just_released.iter().filter_map(get_key_time) {
		update_skill_aim_time(key, time, duration, queue, &mut active_skill);
	}
}

fn update_skill_aim_time<TTime: Default + Send + Sync + 'static>(
	key: &SlotKey,
	time: &Res<Time<TTime>>,
	duration: &Duration,
	queue: &mut Queue,
	active_skill: &mut Option<&mut Track<Skill<PlayerSkills<SideUnset>, Active>>>,
) {
	if update_aim_time_in_queue(key, time, duration, queue) {
		return;
	}
	update_aim_time_on_active(key, time, duration, active_skill);
}

fn update_aim_time_in_queue<TTime: Default + Send + Sync + 'static>(
	key: &SlotKey,
	time: &Res<Time<TTime>>,
	duration: &Duration,
	queue: &mut Queue,
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
	active_skill: &mut Option<&mut Track<Skill<PlayerSkills<SideUnset>, Active>>>,
) {
	let Some(skill) = active_skill.as_mut() else {
		return;
	};
	if &skill.value.data.0 != key {
		return;
	}
	skill.value.cast.aim = time.elapsed() - *duration;
}

fn get_queued_skill<'a>(
	key: &SlotKey,
	queue: &'a mut Queue,
) -> Option<&'a mut Skill<PlayerSkills<SideUnset>, Queued>> {
	queue.0.iter_mut().rev().find(|skill| &skill.data.0 == key)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{Item, Queue, Slot, SlotKey, Slots, Track},
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
	use std::{collections::VecDeque, time::Duration};

	#[derive(Resource, Default)]
	struct _Input(Input);

	fn setup() -> App {
		let mut app = App::new_single_threaded([Update]);
		app.init_resource::<Time<Real>>();
		app.init_resource::<_Input>();
		app.tick_time(Duration::from_millis(42));
		app.add_systems(
			Update,
			(move |input: Res<_Input>| input.0.clone()).pipe(skill_controller::<Real>),
		);

		app
	}

	#[test]
	fn enqueue_first_skill() {
		let mut app = setup();
		let skill = Skill {
			name: "my skill",
			..default()
		};
		let agent = app
			.world
			.spawn((
				Slots(HashMap::from([(
					SlotKey::Hand(Side::Main),
					Slot {
						entity: Entity::from_raw(42),
						item: Some(Item {
							skill: Some(skill.clone()),
							..default()
						}),
						combo_skill: None,
					},
				)])),
				Queue::default(),
			))
			.id();

		app.world.resource_mut::<_Input>().0.just_pressed = vec![SlotKey::Hand(Side::Main)];
		app.update();

		let agent = app.world.entity(agent);
		let queued_skill = skill.with(Queued(SlotKey::Hand(Side::Main)));

		assert_eq!(
			Some(&Queue(VecDeque::from([queued_skill]))),
			agent.get::<Queue>()
		);
	}

	#[test]
	fn enqueue_second_skill() {
		let mut app = setup();
		let first_skill = Skill::default().with(Queued(SlotKey::Hand(Side::Off)));
		let skill = Skill {
			name: "my skill",
			..default()
		};
		let agent = app
			.world
			.spawn((
				Slots(HashMap::from([(
					SlotKey::Hand(Side::Main),
					Slot {
						entity: Entity::from_raw(42),
						item: Some(Item {
							skill: Some(skill.clone()),
							..default()
						}),
						combo_skill: None,
					},
				)])),
				Queue(VecDeque::from([first_skill.clone()])),
			))
			.id();

		app.world.resource_mut::<_Input>().0.just_pressed = vec![SlotKey::Hand(Side::Main)];
		app.update();

		let agent = app.world.entity(agent);
		let queued_skill = skill.with(Queued(SlotKey::Hand(Side::Main)));

		assert_eq!(
			Some(&Queue(VecDeque::from([first_skill, queued_skill,]))),
			agent.get::<Queue>()
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
				Queue::default(),
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
			Some(&Queue(VecDeque::from([Skill {
				name: "main",
				cast: Cast {
					aim: Duration::from_millis(100),
					..default()
				},
				..default()
			}
			.with(Queued(SlotKey::Hand(Side::Main)))]))),
			agent.get::<Queue>()
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
				Queue::default(),
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
			Some(&Queue(VecDeque::from([Skill {
				name: "main",
				cast: Cast {
					aim: Duration::from_millis(300),
					..default()
				},
				..default()
			}
			.with(Queued(SlotKey::Hand(Side::Main)))]))),
			agent.get::<Queue>()
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
				Queue(VecDeque::from([
					Skill::default().with(Queued(SlotKey::Hand(Side::Off)))
				])),
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
			Some(&Queue(VecDeque::from([
				Skill::default().with(Queued(SlotKey::Hand(Side::Off))),
				Skill {
					name: "main",
					cast: Cast {
						aim: Duration::from_millis(100),
						..default()
					},
					..default()
				}
				.with(Queued(SlotKey::Hand(Side::Main))),
			]))),
			agent.get::<Queue>()
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
				Queue::default(),
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
			Some(&Queue(VecDeque::from([
				Skill {
					name: "main",
					cast: Cast {
						aim: Duration::from_millis(200),
						..default()
					},
					..default()
				}
				.with(Queued(SlotKey::Hand(Side::Main))),
				Skill {
					name: "off",
					cast: Cast {
						aim: Duration::from_millis(100),
						..default()
					},
					..default()
				}
				.with(Queued(SlotKey::Hand(Side::Off)))
			]))),
			agent.get::<Queue>()
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
				Queue(VecDeque::from([Skill {
					name: "other",
					..default()
				}
				.with(Queued(SlotKey::Hand(Side::Main)))])),
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
			Some(&Queue(VecDeque::from([
				Skill {
					name: "other",
					..default()
				}
				.with(Queued(SlotKey::Hand(Side::Main))),
				Skill {
					name: "main",
					cast: Cast {
						aim: Duration::from_millis(100),
						..default()
					},
					..default()
				}
				.with(Queued(SlotKey::Hand(Side::Main)))
			]))),
			agent.get::<Queue>()
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
				Queue::default(),
			))
			.id();

		app.world.resource_mut::<_Input>().0.just_pressed = vec![SlotKey::Hand(Side::Main)];
		app.update();

		app.world.entity_mut(agent).get_mut::<Queue>().unwrap().0 = VecDeque::from([]);
		app.world.entity_mut(agent).insert(Track::new(
			Skill::default().with(Active(SlotKey::Hand(Side::Main))),
		));

		app.tick_time(Duration::from_millis(100));
		app.world.resource_mut::<_Input>().0.just_pressed = vec![];
		app.world.resource_mut::<_Input>().0.just_released = vec![SlotKey::Hand(Side::Main)];
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&Track::new(
				Skill {
					cast: Cast {
						aim: Duration::from_millis(100),
						..default()
					},
					..default()
				}
				.with(Active(SlotKey::Hand(Side::Main))),
			)),
			agent.get::<Track<Skill<PlayerSkills<SideUnset>, Active>>>()
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
				Queue::default(),
			))
			.id();

		app.world.resource_mut::<_Input>().0.just_pressed = vec![SlotKey::Hand(Side::Main)];
		app.update();

		app.world.entity_mut(agent).get_mut::<Queue>().unwrap().0 = VecDeque::from([]);
		app.world.entity_mut(agent).insert(Track::new(
			Skill::default().with(Active(SlotKey::Hand(Side::Off))),
		));

		app.tick_time(Duration::from_millis(100));
		app.world.resource_mut::<_Input>().0.just_pressed = vec![];
		app.world.resource_mut::<_Input>().0.just_released = vec![SlotKey::Hand(Side::Main)];
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&Track::new(
				Skill::default().with(Active(SlotKey::Hand(Side::Off))),
			)),
			agent.get::<Track<Skill<PlayerSkills<SideUnset>, Active>>>()
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
				Queue::default(),
			))
			.id();

		app.world.resource_mut::<_Input>().0.just_pressed = vec![SlotKey::Hand(Side::Main)];
		app.update();

		app.world.entity_mut(agent).get_mut::<Queue>().unwrap().0 = VecDeque::from([]);
		app.world.entity_mut(agent).insert(Track::new(
			Skill::default().with(Active(SlotKey::Hand(Side::Main))),
		));

		app.tick_time(Duration::from_millis(100));
		app.world.resource_mut::<_Input>().0.just_pressed = vec![];
		app.update();

		app.tick_time(Duration::from_millis(100));
		app.world.resource_mut::<_Input>().0.just_released = vec![SlotKey::Hand(Side::Main)];
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&Track::new(
				Skill {
					cast: Cast {
						aim: Duration::from_millis(200),
						..default()
					},
					..default()
				}
				.with(Active(SlotKey::Hand(Side::Main))),
			)),
			agent.get::<Track<Skill<PlayerSkills<SideUnset>, Active>>>()
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
				Queue::default(),
			))
			.id();

		app.world.resource_mut::<_Input>().0.just_pressed = vec![SlotKey::Hand(Side::Main)];
		app.update();

		app.world.entity_mut(agent).insert(Track::new(
			Skill::default().with(Active(SlotKey::Hand(Side::Main))),
		));

		app.tick_time(Duration::from_millis(100));
		app.world.resource_mut::<_Input>().0.just_pressed = vec![];
		app.world.resource_mut::<_Input>().0.just_released = vec![SlotKey::Hand(Side::Main)];
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			(
				Some(&Queue(VecDeque::from([Skill {
					name: "main",
					cast: Cast {
						aim: Duration::from_millis(100),
						..default()
					},
					..default()
				}
				.with(Queued(SlotKey::Hand(Side::Main)))]))),
				Some(&Track::new(
					Skill::default().with(Active(SlotKey::Hand(Side::Main))),
				))
			),
			(
				agent.get::<Queue>(),
				agent.get::<Track<Skill<PlayerSkills<SideUnset>, Active>>>()
			)
		);
	}
}
