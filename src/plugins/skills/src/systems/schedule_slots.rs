use crate::{
	components::{Schedule, SlotKey, Slots},
	resources::SlotMap,
	skill::Skill,
	traits::{InputState, ShouldEnqueue},
};
use bevy::{
	ecs::system::Resource,
	prelude::{Commands, Component, Entity, Query, Res, With},
	time::{Real, Time},
};
use std::{fmt::Debug, hash::Hash, time::Duration};

#[derive(Component, Debug, PartialEq)]
pub struct CurrentlyScheduling {
	slot: SlotKey,
	time_stamp: Duration,
}

pub(crate) fn schedule_slots<
	TKey: Copy + Eq + Hash + Debug + Send + Sync + 'static,
	TAgent: Component,
	TInput: InputState<TKey> + Resource,
	TShouldEnqueue: ShouldEnqueue + Resource,
>(
	input: Res<TInput>,
	should_enqueue: Res<TShouldEnqueue>,
	slot_map: Res<SlotMap<TKey>>,
	agents: Query<(Entity, &Slots, Option<&CurrentlyScheduling>), With<TAgent>>,
	mut commands: Commands,
	time: Res<Time<Real>>,
) {
	if agents.is_empty() {
		return;
	}

	let commands = &mut commands;
	let agents = &agents;
	let time = &time;
	let slot_map = &slot_map;

	let just_pressed = &input.just_pressed_slots(slot_map);
	let pressed = &input.pressed_slots(slot_map);
	let just_released = &input.just_released_slots(slot_map);

	if !just_pressed.is_empty() {
		schedule_new(mode(should_enqueue), commands, agents, just_pressed, time);
	}
	if !pressed.is_empty() {
		update_target(commands, agents, pressed);
	}
	if !just_released.is_empty() {
		schedule_aim(commands, agents, just_released, time);
	}
}

fn mode<TShouldEnqueue: ShouldEnqueue + Resource>(
	should_enqueue: Res<TShouldEnqueue>,
) -> fn((SlotKey, Skill)) -> Schedule {
	match should_enqueue.should_enqueue() {
		true => Schedule::Enqueue,
		false => Schedule::Override,
	}
}

fn schedule_new<TAgent: Component>(
	new_schedule: fn((SlotKey, Skill)) -> Schedule,
	commands: &mut Commands,
	agents: &Query<(Entity, &Slots, Option<&CurrentlyScheduling>), With<TAgent>>,
	just_pressed: &[SlotKey],
	time: &Res<Time<Real>>,
) {
	let valid_agents = agents
		.iter()
		.filter_map(nothing_scheduling_and(just_pressed));
	let time_stamp = time.elapsed();

	for (agent, slot, skill) in valid_agents {
		let mut agent = commands.entity(agent);
		agent.insert(new_schedule((slot, skill)));
		agent.insert(CurrentlyScheduling { slot, time_stamp });
	}
}

fn update_target<TAgent: Component>(
	commands: &mut Commands,
	agents: &Query<(Entity, &Slots, Option<&CurrentlyScheduling>), With<TAgent>>,
	pressed: &[SlotKey],
) {
	let valid_agents = agents
		.iter()
		.filter_map(currently_scheduling_matches(pressed));

	for (agent, _) in valid_agents {
		commands.entity(agent).insert(Schedule::UpdateTarget);
	}
}

fn schedule_aim<TAgent: Component>(
	commands: &mut Commands,
	agents: &Query<(Entity, &Slots, Option<&CurrentlyScheduling>), With<TAgent>>,
	just_released: &[SlotKey],
	time: &Res<Time<Real>>,
) {
	let valid_agents = agents
		.iter()
		.filter_map(currently_scheduling_matches(just_released));
	let elapsed = time.elapsed();

	for (agent, currently_scheduling) in valid_agents {
		let aim_duration = elapsed - currently_scheduling.time_stamp;
		let mut agent = commands.entity(agent);
		agent.insert(Schedule::StopAimAfter(aim_duration));
		agent.remove::<CurrentlyScheduling>();
	}
}

fn nothing_scheduling_and<'a>(
	active_keys: &'a [SlotKey],
) -> impl Fn((Entity, &Slots, Option<&'a CurrentlyScheduling>)) -> Option<(Entity, SlotKey, Skill)> + 'a
{
	|(agent, slots, currently_scheduling): (Entity, &Slots, Option<&'a CurrentlyScheduling>)| {
		if currently_scheduling.is_some() {
			return None;
		};
		let (slot, skill) = get_skill_and_slot(slots, active_keys)?;
		Some((agent, slot, skill))
	}
}

fn currently_scheduling_matches<'a>(
	active_keys: &'a [SlotKey],
) -> impl Fn(
	(Entity, &Slots, Option<&'a CurrentlyScheduling>),
) -> Option<(Entity, &'a CurrentlyScheduling)>
       + 'a {
	|(agent, _, currently_scheduling): (Entity, &Slots, Option<&'a CurrentlyScheduling>)| {
		let currently_scheduling = currently_scheduling?;
		if !active_keys.contains(&currently_scheduling.slot) {
			return None;
		}
		Some((agent, currently_scheduling))
	}
}

fn get_skill_and_slot(slots: &Slots, slot_keys: &[SlotKey]) -> Option<(SlotKey, Skill)> {
	slots
		.0
		.iter()
		.filter(|(sk, ..)| slot_keys.contains(sk))
		.find_map(|(k, s)| Some((*k, s.combo_skill.clone().or(s.item.clone()?.skill)?)))
}

#[cfg(test)]
mod tests {
	use crate::components::{Item, Slot};

	use super::*;
	use bevy::{
		prelude::{default, App, Component, Entity, KeyCode, Update},
		time::{Real, Time},
	};
	use common::{components::Side, test_tools::utils::TickTime};
	use mockall::{mock, predicate::eq};

	#[derive(Component)]
	struct _Agent;

	#[derive(Default, Resource)]
	struct _Input {
		pub mock: Mock_Input,
	}

	impl InputState<KeyCode> for _Input {
		fn just_pressed_slots(&self, map: &SlotMap<KeyCode>) -> Vec<SlotKey> {
			self.mock.just_pressed_slots(map)
		}

		fn pressed_slots(&self, map: &SlotMap<KeyCode>) -> Vec<SlotKey> {
			self.mock.pressed_slots(map)
		}

		fn just_released_slots(&self, map: &SlotMap<KeyCode>) -> Vec<SlotKey> {
			self.mock.just_released_slots(map)
		}
	}

	impl ShouldEnqueue for _Input {
		fn should_enqueue(&self) -> bool {
			self.mock.should_enqueue()
		}
	}

	mock! {
		_Input {}
		impl InputState<KeyCode> for _Input {
			fn just_pressed_slots(&self, map: &SlotMap<KeyCode>) -> Vec<SlotKey> {}
			fn pressed_slots(&self, map: &SlotMap<KeyCode>) -> Vec<SlotKey> {}
			fn just_released_slots(&self, map: &SlotMap<KeyCode>) -> Vec<SlotKey> {}
		}
		impl ShouldEnqueue for _Input {
			fn should_enqueue(&self) -> bool {}
		}
	}

	fn setup(keys: Keys) -> App {
		let mut app = App::new();

		app.init_resource::<Time<Real>>()
			.init_resource::<SlotMap<KeyCode>>()
			.add_systems(Update, schedule_slots::<KeyCode, _Agent, _Input, _Input>);

		app.insert_resource(get_input(keys));

		app
	}

	#[derive(Default)]
	struct Keys {
		just_pressed: Vec<SlotKey>,
		pressed: Vec<SlotKey>,
		just_released: Vec<SlotKey>,
		should_enqueue: bool,
	}

	fn get_input(keys: Keys) -> _Input {
		let Keys {
			just_pressed,
			pressed,
			just_released,
			should_enqueue,
		} = keys;
		let mut input = _Input::default();

		input
			.mock
			.expect_just_pressed_slots()
			.return_const(just_pressed);
		input.mock.expect_pressed_slots().return_const(pressed);
		input
			.mock
			.expect_just_released_slots()
			.return_const(just_released);
		input
			.mock
			.expect_should_enqueue()
			.return_const(should_enqueue);

		input
	}

	#[test]
	fn set_override() {
		let mut app = setup(Keys {
			just_pressed: vec![SlotKey::Hand(Side::Main)],
			..default()
		});
		let slots = Slots(
			[(
				SlotKey::Hand(Side::Main),
				Slot {
					entity: Entity::from_raw(42),
					item: Some(Item {
						skill: Some(Skill {
							name: "skill",
							..default()
						}),
						..default()
					}),
					combo_skill: None,
				},
			)]
			.into(),
		);
		let agent = app.world.spawn((_Agent, slots)).id();

		app.update();

		let agent = app.world.entity(agent);
		let schedule = agent.get::<Schedule>();
		let currently_scheduling = agent.get::<CurrentlyScheduling>();

		assert_eq!(
			(
				Some(&Schedule::Override((
					SlotKey::Hand(Side::Main),
					Skill {
						name: "skill",
						..default()
					}
				))),
				Some(&CurrentlyScheduling {
					slot: SlotKey::Hand(Side::Main),
					time_stamp: app.world.resource::<Time<Real>>().elapsed()
				})
			),
			(schedule, currently_scheduling)
		);
	}

	#[test]
	fn set_override_combo() {
		let mut app = setup(Keys {
			just_pressed: vec![SlotKey::Hand(Side::Main)],
			..default()
		});
		let slots = Slots(
			[(
				SlotKey::Hand(Side::Main),
				Slot {
					entity: Entity::from_raw(42),
					item: Some(Item {
						skill: Some(Skill {
							name: "skill",
							..default()
						}),
						..default()
					}),
					combo_skill: Some(Skill {
						name: "combo skill",
						..default()
					}),
				},
			)]
			.into(),
		);
		let agent = app.world.spawn((_Agent, slots)).id();

		app.update();

		let agent = app.world.entity(agent);
		let schedule = agent.get::<Schedule>();

		assert_eq!(
			Some(&Schedule::Override((
				SlotKey::Hand(Side::Main),
				Skill {
					name: "combo skill",
					..default()
				}
			))),
			schedule
		);
	}

	#[test]
	fn do_not_set_when_no_skill() {
		let mut app = setup(Keys {
			just_pressed: vec![SlotKey::Hand(Side::Main)],
			..default()
		});
		let slots = Slots(
			[(
				SlotKey::Hand(Side::Main),
				Slot {
					entity: Entity::from_raw(42),
					item: Some(Item::default()),
					combo_skill: None,
				},
			)]
			.into(),
		);
		let agent = app.world.spawn((_Agent, slots)).id();

		app.update();

		let agent = app.world.entity(agent);
		let schedule = agent.get::<Schedule>();

		assert_eq!(None, schedule);
	}

	#[test]
	fn do_not_set_when_currently_scheduling_agent() {
		let mut app = setup(Keys {
			just_pressed: vec![SlotKey::Hand(Side::Main)],
			..default()
		});
		let slots = Slots(
			[(
				SlotKey::Hand(Side::Main),
				Slot {
					entity: Entity::from_raw(42),
					item: Some(Item {
						skill: Some(Skill {
							name: "skill",
							..default()
						}),
						..default()
					}),
					combo_skill: None,
				},
			)]
			.into(),
		);
		let agent = app
			.world
			.spawn((
				_Agent,
				slots,
				CurrentlyScheduling {
					slot: SlotKey::Hand(Side::Main),
					time_stamp: Duration::ZERO,
				},
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);
		let schedule = agent.get::<Schedule>();

		assert_eq!(None, schedule);
	}

	#[test]
	fn do_not_set_when_no_agent() {
		let mut app = setup(Keys {
			just_pressed: vec![SlotKey::Hand(Side::Main)],
			..default()
		});
		let slots = Slots(
			[(
				SlotKey::Hand(Side::Main),
				Slot {
					entity: Entity::from_raw(42),
					item: Some(Item {
						skill: Some(Skill {
							name: "skill",
							..default()
						}),
						..default()
					}),
					combo_skill: None,
				},
			)]
			.into(),
		);
		let agent = app.world.spawn(slots).id();

		app.update();

		let agent = app.world.entity(agent);
		let schedule = agent.get::<Schedule>();

		assert_eq!(None, schedule);
	}

	#[test]
	fn set_when_2nd_pressed_key_matches_a_slot() {
		let mut app = setup(Keys {
			just_pressed: vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Main)],
			..default()
		});
		let slots = Slots(
			[(
				SlotKey::Hand(Side::Main),
				Slot {
					entity: Entity::from_raw(42),
					item: Some(Item {
						skill: Some(Skill {
							name: "skill",
							..default()
						}),
						..default()
					}),
					combo_skill: None,
				},
			)]
			.into(),
		);
		let agent = app.world.spawn((_Agent, slots)).id();

		app.tick_time(Duration::from_millis(1000));
		app.update();

		let agent = app.world.entity(agent);
		let schedule = agent.get::<Schedule>();
		let currently_scheduling = agent.get::<CurrentlyScheduling>();

		assert_eq!(
			(
				Some(&Schedule::Override((
					SlotKey::Hand(Side::Main),
					Skill {
						name: "skill",
						..default()
					}
				))),
				Some(&CurrentlyScheduling {
					slot: SlotKey::Hand(Side::Main),
					time_stamp: app.world.resource::<Time<Real>>().elapsed()
				})
			),
			(schedule, currently_scheduling)
		);
	}

	#[test]
	fn set_enqueue() {
		let mut app = setup(Keys {
			just_pressed: vec![SlotKey::Hand(Side::Main)],
			should_enqueue: true,
			..default()
		});
		let slots = Slots(
			[(
				SlotKey::Hand(Side::Main),
				Slot {
					entity: Entity::from_raw(42),
					item: Some(Item {
						skill: Some(Skill {
							name: "skill",
							..default()
						}),
						..default()
					}),
					combo_skill: None,
				},
			)]
			.into(),
		);
		let agent = app.world.spawn((_Agent, slots)).id();

		app.tick_time(Duration::from_millis(100));
		app.update();

		let agent = app.world.entity(agent);
		let schedule = agent.get::<Schedule>();
		let currently_scheduling = agent.get::<CurrentlyScheduling>();

		assert_eq!(
			(
				Some(&Schedule::Enqueue((
					SlotKey::Hand(Side::Main),
					Skill {
						name: "skill",
						..default()
					}
				))),
				Some(&CurrentlyScheduling {
					slot: SlotKey::Hand(Side::Main),
					time_stamp: app.world.resource::<Time<Real>>().elapsed()
				})
			),
			(schedule, currently_scheduling)
		);
	}

	#[test]
	fn set_enqueue_combo_skill() {
		let mut app = setup(Keys {
			just_pressed: vec![SlotKey::Hand(Side::Main)],
			should_enqueue: true,
			..default()
		});
		let slots = Slots(
			[(
				SlotKey::Hand(Side::Main),
				Slot {
					entity: Entity::from_raw(42),
					item: Some(Item {
						skill: Some(Skill {
							name: "skill",
							..default()
						}),
						..default()
					}),
					combo_skill: Some(Skill {
						name: "combo skill",
						..default()
					}),
				},
			)]
			.into(),
		);
		let agent = app.world.spawn((_Agent, slots)).id();

		app.update();

		let agent = app.world.entity(agent);
		let schedule = agent.get::<Schedule>();

		assert_eq!(
			Some(&Schedule::Enqueue((
				SlotKey::Hand(Side::Main),
				Skill {
					name: "combo skill",
					..default()
				}
			))),
			schedule
		);
	}

	#[test]
	fn set_aim() {
		let mut app = setup(Keys {
			just_released: vec![SlotKey::Hand(Side::Main)],
			..default()
		});
		let slots = Slots(
			[(
				SlotKey::Hand(Side::Main),
				Slot {
					entity: Entity::from_raw(42),
					item: Some(Item {
						skill: Some(Skill {
							name: "skill",
							..default()
						}),
						..default()
					}),
					combo_skill: None,
				},
			)]
			.into(),
		);
		let time_stamp = app.world.resource::<Time<Real>>().elapsed();
		let agent = app
			.world
			.spawn((
				_Agent,
				slots,
				CurrentlyScheduling {
					slot: SlotKey::Hand(Side::Main),
					time_stamp,
				},
			))
			.id();

		app.tick_time(Duration::from_micros(500));
		app.update();

		let agent = app.world.entity(agent);
		let schedule = agent.get::<Schedule>();
		let currently_scheduling = agent.get::<CurrentlyScheduling>();

		assert_eq!(
			(
				Some(&Schedule::StopAimAfter(Duration::from_micros(500))),
				None
			),
			(schedule, currently_scheduling)
		);
	}

	#[test]
	fn do_not_set_aim_when_not_matching_previously_scheduled_slot() {
		let mut app = setup(Keys {
			just_released: vec![SlotKey::Hand(Side::Main)],
			..default()
		});
		let slots = Slots(
			[(
				SlotKey::Hand(Side::Main),
				Slot {
					entity: Entity::from_raw(42),
					item: Some(Item {
						skill: Some(Skill {
							name: "skill",
							..default()
						}),
						..default()
					}),
					combo_skill: None,
				},
			)]
			.into(),
		);
		let agent = app.world.spawn((_Agent, slots)).id();

		app.update();

		let agent = app.world.entity(agent);
		let schedule = agent.get::<Schedule>();

		assert_eq!(None, schedule);
	}

	#[test]
	fn set_aim_with_2nd_just_released_key() {
		let mut app = setup(Keys {
			just_released: vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Main)],
			..default()
		});
		let slots = Slots(
			[(
				SlotKey::Hand(Side::Main),
				Slot {
					entity: Entity::from_raw(42),
					item: Some(Item {
						skill: Some(Skill {
							name: "skill",
							..default()
						}),
						..default()
					}),
					combo_skill: None,
				},
			)]
			.into(),
		);
		let time_stamp = app.world.resource::<Time<Real>>().elapsed();
		let agent = app
			.world
			.spawn((
				_Agent,
				slots,
				CurrentlyScheduling {
					slot: SlotKey::Hand(Side::Main),
					time_stamp,
				},
			))
			.id();

		app.tick_time(Duration::from_micros(500));
		app.update();

		let agent = app.world.entity(agent);
		let schedule = agent.get::<Schedule>();

		assert_eq!(
			Some(&Schedule::StopAimAfter(Duration::from_micros(500))),
			schedule
		);
	}

	#[test]
	fn update_target_on_hold() {
		let mut app = setup(Keys {
			pressed: vec![SlotKey::Hand(Side::Main)],
			..default()
		});
		let slots = Slots(
			[(
				SlotKey::Hand(Side::Main),
				Slot {
					entity: Entity::from_raw(42),
					item: Some(Item {
						skill: Some(Skill {
							name: "skill",
							..default()
						}),
						..default()
					}),
					combo_skill: None,
				},
			)]
			.into(),
		);
		let time_stamp = app.world.resource::<Time<Real>>().elapsed();
		let agent = app
			.world
			.spawn((
				_Agent,
				slots,
				CurrentlyScheduling {
					slot: SlotKey::Hand(Side::Main),
					time_stamp,
				},
			))
			.id();

		app.tick_time(Duration::from_micros(500));
		app.update();

		let agent = app.world.entity(agent);
		let schedule = agent.get::<Schedule>();

		assert_eq!(Some(&Schedule::UpdateTarget), schedule);
	}

	#[test]
	fn do_not_update_target_when_not_matching_currently_scheduling() {
		let mut app = setup(Keys {
			pressed: vec![SlotKey::Hand(Side::Main)],
			..default()
		});
		let slots = Slots(
			[(
				SlotKey::Hand(Side::Main),
				Slot {
					entity: Entity::from_raw(42),
					item: Some(Item {
						skill: Some(Skill {
							name: "skill",
							..default()
						}),
						..default()
					}),
					combo_skill: None,
				},
			)]
			.into(),
		);
		let time_stamp = app.world.resource::<Time<Real>>().elapsed();
		let agent = app
			.world
			.spawn((
				_Agent,
				slots,
				CurrentlyScheduling {
					slot: SlotKey::Hand(Side::Off),
					time_stamp,
				},
			))
			.id();

		app.tick_time(Duration::from_micros(500));
		app.update();

		let agent = app.world.entity(agent);
		let schedule = agent.get::<Schedule>();

		assert_eq!(None, schedule);
	}

	#[test]
	fn update_target_with_2nd_hold_key() {
		let mut app = setup(Keys {
			pressed: vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Main)],
			..default()
		});
		let slots = Slots(
			[(
				SlotKey::Hand(Side::Main),
				Slot {
					entity: Entity::from_raw(42),
					item: Some(Item {
						skill: Some(Skill {
							name: "skill",
							..default()
						}),
						..default()
					}),
					combo_skill: None,
				},
			)]
			.into(),
		);
		let time_stamp = app.world.resource::<Time<Real>>().elapsed();
		let agent = app
			.world
			.spawn((
				_Agent,
				slots,
				CurrentlyScheduling {
					slot: SlotKey::Hand(Side::Main),
					time_stamp,
				},
			))
			.id();

		app.tick_time(Duration::from_micros(500));
		app.update();

		let agent = app.world.entity(agent);
		let schedule = agent.get::<Schedule>();

		assert_eq!(Some(&Schedule::UpdateTarget), schedule);
	}

	#[test]
	fn call_just_pressed_with_slot_map() {
		let mut app = App::new();
		let slot_map = SlotMap::new([(KeyCode::A, SlotKey::Hand(Side::Main), "")]);

		app.init_resource::<Time<Real>>();
		app.insert_resource(slot_map.clone());
		app.add_systems(Update, schedule_slots::<KeyCode, _Agent, _Input, _Input>);

		let mut input = _Input::default();
		input
			.mock
			.expect_just_pressed_slots()
			.times(1)
			.with(eq(slot_map))
			.return_const(vec![]);
		input.mock.expect_pressed_slots().return_const(vec![]);
		input.mock.expect_just_released_slots().return_const(vec![]);
		input.mock.expect_should_enqueue().return_const(false);
		app.insert_resource(input);

		app.world.spawn((_Agent, Slots::default()));
		app.update();
	}

	#[test]
	fn call_pressed_with_slot_map() {
		let mut app = App::new();
		let slot_map = SlotMap::new([(KeyCode::A, SlotKey::Hand(Side::Main), "")]);

		app.init_resource::<Time<Real>>();
		app.insert_resource(slot_map.clone());
		app.add_systems(Update, schedule_slots::<KeyCode, _Agent, _Input, _Input>);

		let mut input = _Input::default();
		input.mock.expect_just_pressed_slots().return_const(vec![]);
		input
			.mock
			.expect_pressed_slots()
			.times(1)
			.with(eq(slot_map))
			.return_const(vec![]);
		input.mock.expect_just_released_slots().return_const(vec![]);
		input.mock.expect_should_enqueue().return_const(false);
		app.insert_resource(input);

		app.world.spawn((_Agent, Slots::default()));
		app.update();
	}

	#[test]
	fn call_just_released_with_slot_map() {
		let mut app = App::new();
		let slot_map = SlotMap::new([(KeyCode::A, SlotKey::Hand(Side::Main), "")]);

		app.init_resource::<Time<Real>>();
		app.insert_resource(slot_map.clone());
		app.add_systems(Update, schedule_slots::<KeyCode, _Agent, _Input, _Input>);

		let mut input = _Input::default();
		input.mock.expect_just_pressed_slots().return_const(vec![]);
		input.mock.expect_pressed_slots().return_const(vec![]);
		input
			.mock
			.expect_just_released_slots()
			.times(1)
			.with(eq(slot_map))
			.return_const(vec![]);
		input.mock.expect_should_enqueue().return_const(false);
		app.insert_resource(input);

		app.world.spawn((_Agent, Slots::default()));
		app.update();
	}

	#[test]
	fn call_none_when_no_valid_entity_with_components() {
		let mut app = App::new();

		app.init_resource::<Time<Real>>();
		app.init_resource::<SlotMap<KeyCode>>();
		app.add_systems(Update, schedule_slots::<KeyCode, _Agent, _Input, _Input>);

		let mut input = _Input::default();
		input
			.mock
			.expect_just_pressed_slots()
			.times(0)
			.return_const(vec![]);
		input
			.mock
			.expect_pressed_slots()
			.times(0)
			.return_const(vec![]);
		input
			.mock
			.expect_just_released_slots()
			.times(0)
			.return_const(vec![]);
		input
			.mock
			.expect_should_enqueue()
			.times(0)
			.return_const(false);
		app.insert_resource(input);

		app.update();
	}
}
