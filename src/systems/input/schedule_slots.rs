use crate::{
	components::{Schedule, SlotKey, Slots},
	resources::SlotMap,
	skill::Skill,
	states::MouseContext,
};
use bevy::{
	ecs::{schedule::State, system::Local},
	prelude::{Commands, Component, Entity, Input, KeyCode, Query, Res, With},
	time::{Real, Time},
};
use std::{fmt::Debug, hash::Hash, time::Duration};

// FIXME: Needs to be separated into several systems?
#[allow(clippy::too_many_arguments)]
pub fn schedule_slots<TKey: Copy + Eq + Hash + Debug + Send + Sync, TAgent: Component>(
	input: Res<Input<TKey>>,
	mouse_context: Res<State<MouseContext<TKey>>>,
	keys: Res<Input<KeyCode>>,
	slot_map: Res<SlotMap<TKey>>,
	agents: Query<(Entity, &Slots), With<TAgent>>,
	commands: Commands,
	added_new: Local<Option<(SlotKey, Duration)>>,
	time: Res<Time<Real>>,
) {
	let pressed = input
		.get_just_pressed()
		.find_map(|key| slot_map.slots.get(key))
		.cloned()
		.or_else(|| just_triggered_mouse_context(&mouse_context, &slot_map));
	let released = input
		.get_just_released()
		.find_map(|key| slot_map.slots.get(key))
		.cloned()
		.or_else(|| just_released_mouse_context(&mouse_context, &slot_map));

	match (pressed, released) {
		(Some(pressed), ..) => schedule_new(mode(keys), commands, added_new, agents, pressed, time),
		(.., Some(released)) => schedule_transition(commands, added_new, agents, released, time),
		_ => (),
	};
}

fn mode(keys: Res<Input<KeyCode>>) -> fn((SlotKey, Skill)) -> Schedule {
	match keys.pressed(KeyCode::ShiftLeft) {
		true => Schedule::Enqueue,
		false => Schedule::Override,
	}
}

fn schedule_new<TAgent: Component>(
	new_schedule: fn((SlotKey, Skill)) -> Schedule,
	mut commands: Commands,
	mut last_added: Local<Option<(SlotKey, Duration)>>,
	agents: Query<(Entity, &Slots), With<TAgent>>,
	pressed: SlotKey,
	time: Res<Time<Real>>,
) {
	for (agent, slots) in &agents {
		if let Some(skill) = get_skill(slots, pressed) {
			commands.entity(agent).insert(new_schedule(skill));
		}
	}
	*last_added = Some((pressed, time.elapsed()));
}

fn schedule_transition<TAgent: Component>(
	mut commands: Commands,
	last_added: Local<Option<(SlotKey, Duration)>>,
	agents: Query<(Entity, &Slots), With<TAgent>>,
	released: SlotKey,
	time: Res<Time<Real>>,
) {
	let Some((add_slot, add_time)) = *last_added else {
		return;
	};
	if add_slot != released {
		return;
	}
	let pre_transition_time = time.elapsed() - add_time;
	for (agent, ..) in &agents {
		commands
			.entity(agent)
			.insert(Schedule::TransitionAfter(pre_transition_time));
	}
}

fn just_triggered_mouse_context<TKey: Copy + Eq + Hash + Debug + Send + Sync>(
	mouse_context: &Res<State<MouseContext<TKey>>>,
	slot_map: &Res<SlotMap<TKey>>,
) -> Option<SlotKey> {
	let MouseContext::JustTriggered(key) = mouse_context.get() else {
		return None;
	};
	slot_map.slots.get(key).copied()
}

fn just_released_mouse_context<TKey: Copy + Eq + Hash + Debug + Send + Sync>(
	mouse_context: &Res<State<MouseContext<TKey>>>,
	slot_map: &Res<SlotMap<TKey>>,
) -> Option<SlotKey> {
	let MouseContext::JustReleased(key) = mouse_context.get() else {
		return None;
	};
	slot_map.slots.get(key).copied()
}

fn get_skill(slots: &Slots, slot_key: SlotKey) -> Option<(SlotKey, Skill)> {
	slots
		.0
		.iter()
		.filter(|(sk, ..)| sk == &&slot_key)
		.find_map(|(k, s)| Some((*k, s.combo_skill.clone().or(s.item.clone()?.skill)?)))
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{Item, Schedule, Side, Slot, SlotKey, Slots},
		resources::SlotMap,
		test_tools::utils::TickTime,
	};
	use bevy::{
		ecs::schedule::NextState,
		prelude::{default, App, Component, Entity, Input, KeyCode, MouseButton, Update},
		time::{Real, Time},
	};

	#[derive(Component)]
	struct TestAgent;

	fn setup() -> App {
		let mut app = App::new();

		app.add_state::<MouseContext<MouseButton>>()
			.init_resource::<Time<Real>>()
			.init_resource::<Input<MouseButton>>()
			.init_resource::<Input<KeyCode>>()
			.init_resource::<SlotMap<MouseButton>>()
			.add_systems(Update, schedule_slots::<MouseButton, TestAgent>);

		app
	}

	#[test]
	fn set_override() {
		let mut app = setup();
		let slots = Slots(
			[(
				SlotKey::Legs,
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
		let agent = app.world.spawn((TestAgent, slots)).id();

		app.world
			.resource_mut::<SlotMap<MouseButton>>()
			.slots
			.insert(MouseButton::Right, SlotKey::Legs);
		app.world
			.resource_mut::<Input<MouseButton>>()
			.press(MouseButton::Right);

		app.update();

		let agent = app.world.entity(agent);
		let schedule = agent.get::<Schedule>();

		assert_eq!(
			Some(&Schedule::Override((
				SlotKey::Legs,
				Skill {
					name: "skill",
					..default()
				}
			))),
			schedule
		);
	}

	#[test]
	fn set_override_alternative() {
		let mut app = setup();
		let slots = Slots(
			[(
				SlotKey::Legs,
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
						name: "alternative skill",
						..default()
					}),
				},
			)]
			.into(),
		);
		let agent = app.world.spawn((TestAgent, slots)).id();

		app.world
			.resource_mut::<SlotMap<MouseButton>>()
			.slots
			.insert(MouseButton::Right, SlotKey::Legs);
		app.world
			.resource_mut::<Input<MouseButton>>()
			.press(MouseButton::Right);

		app.update();

		let agent = app.world.entity(agent);
		let schedule = agent.get::<Schedule>();

		assert_eq!(
			Some(&Schedule::Override((
				SlotKey::Legs,
				Skill {
					name: "alternative skill",
					..default()
				}
			))),
			schedule
		);
	}

	#[test]
	fn do_not_set_when_not_on_agent() {
		let mut app = setup();
		let slots = Slots(
			[(
				SlotKey::Legs,
				Slot {
					entity: Entity::from_raw(42),
					item: Some(Item::default()),
					combo_skill: None,
				},
			)]
			.into(),
		);
		let agent = app.world.spawn(slots).id();

		app.world
			.resource_mut::<SlotMap<MouseButton>>()
			.slots
			.insert(MouseButton::Right, SlotKey::Legs);
		app.world
			.resource_mut::<Input<MouseButton>>()
			.press(MouseButton::Right);

		app.update();

		let agent = app.world.entity(agent);
		let schedule = agent.get::<Schedule>();

		assert_eq!(None, schedule);
	}

	#[test]
	fn do_not_set_when_mouse_button_not_correctly_mapped() {
		let mut app = setup();
		let slots = Slots(
			[(
				SlotKey::Legs,
				Slot {
					entity: Entity::from_raw(42),
					item: Some(Item::default()),
					combo_skill: None,
				},
			)]
			.into(),
		);
		let agent = app.world.spawn((TestAgent, slots)).id();

		app.world
			.resource_mut::<SlotMap<MouseButton>>()
			.slots
			.insert(MouseButton::Right, SlotKey::Hand(Side::Off));
		app.world
			.resource_mut::<Input<MouseButton>>()
			.press(MouseButton::Right);

		app.update();

		let agent = app.world.entity(agent);
		let schedule = agent.get::<Schedule>();

		assert_eq!(None, schedule);
	}

	#[test]
	fn set_enqueue() {
		let mut app = setup();
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
		let agent = app.world.spawn((TestAgent, slots)).id();

		app.world
			.resource_mut::<SlotMap<MouseButton>>()
			.slots
			.insert(MouseButton::Left, SlotKey::Hand(Side::Main));
		app.world
			.resource_mut::<Input<MouseButton>>()
			.press(MouseButton::Left);
		app.world
			.resource_mut::<Input<KeyCode>>()
			.press(KeyCode::ShiftLeft);

		app.update();

		let agent = app.world.entity(agent);
		let schedule = agent.get::<Schedule>();

		assert_eq!(
			Some(&Schedule::Enqueue((
				SlotKey::Hand(Side::Main),
				Skill {
					name: "skill",
					..default()
				}
			))),
			schedule
		);
	}

	#[test]
	fn set_enqueue_alternative() {
		let mut app = setup();
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
						name: "alternative skill",
						..default()
					}),
				},
			)]
			.into(),
		);
		let agent = app.world.spawn((TestAgent, slots)).id();

		app.world
			.resource_mut::<SlotMap<MouseButton>>()
			.slots
			.insert(MouseButton::Left, SlotKey::Hand(Side::Main));
		app.world
			.resource_mut::<Input<MouseButton>>()
			.press(MouseButton::Left);
		app.world
			.resource_mut::<Input<KeyCode>>()
			.press(KeyCode::ShiftLeft);

		app.update();

		let agent = app.world.entity(agent);
		let schedule = agent.get::<Schedule>();

		assert_eq!(
			Some(&Schedule::Enqueue((
				SlotKey::Hand(Side::Main),
				Skill {
					name: "alternative skill",
					..default()
				}
			))),
			schedule
		);
	}

	#[test]
	fn do_not_set_when_not_just_mouse_pressed() {
		let mut app = setup();

		app.world
			.resource_mut::<SlotMap<MouseButton>>()
			.slots
			.insert(MouseButton::Right, SlotKey::Legs);
		app.world
			.resource_mut::<Input<MouseButton>>()
			.press(MouseButton::Right);

		app.update();

		let slots = Slots(
			[(
				SlotKey::Legs,
				Slot {
					entity: Entity::from_raw(42),
					item: Some(Item::default()),
					combo_skill: None,
				},
			)]
			.into(),
		);
		let agent = app.world.spawn((TestAgent, slots)).id();
		app.world
			.resource_mut::<Input<MouseButton>>()
			.clear_just_pressed(MouseButton::Right);

		app.update();

		let agent = app.world.entity(agent);
		let schedule = agent.get::<Schedule>();

		assert_eq!(None, schedule);
	}

	#[test]
	fn set_override_via_triggered_mouse_context() {
		let mut app = setup();
		let slots = Slots(
			[(
				SlotKey::Legs,
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
		let agent = app.world.spawn((TestAgent, slots)).id();

		app.world
			.resource_mut::<SlotMap<MouseButton>>()
			.slots
			.insert(MouseButton::Right, SlotKey::Legs);
		app.world
			.resource_mut::<NextState<MouseContext<MouseButton>>>()
			.set(MouseContext::JustTriggered(MouseButton::Right));

		app.update();

		let agent = app.world.entity(agent);
		let schedule = agent.get::<Schedule>();

		assert_eq!(
			Some(&Schedule::Override((
				SlotKey::Legs,
				Skill {
					name: "skill",
					..default()
				}
			))),
			schedule
		);
	}

	#[test]
	fn set_transition() {
		let mut app = setup();
		let slots = Slots(
			[(
				SlotKey::Legs,
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
		let agent = app.world.spawn((TestAgent, slots)).id();

		app.world
			.resource_mut::<SlotMap<MouseButton>>()
			.slots
			.insert(MouseButton::Right, SlotKey::Legs);
		app.world
			.resource_mut::<Input<MouseButton>>()
			.press(MouseButton::Right);

		app.update();

		app.world
			.resource_mut::<Input<MouseButton>>()
			.press(MouseButton::Right);

		app.update();

		app.world
			.resource_mut::<Input<MouseButton>>()
			.clear_just_pressed(MouseButton::Right);
		app.world
			.resource_mut::<Input<MouseButton>>()
			.release(MouseButton::Right);

		app.tick_time(Duration::from_millis(1000));
		app.update();

		let agent = app.world.entity(agent);
		let schedule = agent.get::<Schedule>();

		assert_eq!(
			Some(&Schedule::TransitionAfter(Duration::from_millis(1000))),
			schedule
		);
	}

	#[test]
	fn set_transition_from_mouse_context() {
		let mut app = setup();
		let slots = Slots(
			[(
				SlotKey::Legs,
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
		let agent = app.world.spawn((TestAgent, slots)).id();

		app.world
			.resource_mut::<SlotMap<MouseButton>>()
			.slots
			.insert(MouseButton::Right, SlotKey::Legs);
		app.world
			.resource_mut::<Input<MouseButton>>()
			.press(MouseButton::Right);

		app.update();

		app.world
			.resource_mut::<Input<MouseButton>>()
			.clear_just_pressed(MouseButton::Right);
		app.world
			.resource_mut::<NextState<MouseContext<MouseButton>>>()
			.set(MouseContext::JustReleased(MouseButton::Right));

		app.tick_time(Duration::from_millis(400));
		app.update();

		let agent = app.world.entity(agent);
		let schedule = agent.get::<Schedule>();

		assert_eq!(
			Some(&Schedule::TransitionAfter(Duration::from_millis(400))),
			schedule
		);
	}

	#[test]
	fn do_not_set_transition_when_not_previously_set() {
		let mut app = setup();
		let slots = Slots(
			[(
				SlotKey::Legs,
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
		let agent = app.world.spawn((TestAgent, slots)).id();

		app.world
			.resource_mut::<SlotMap<MouseButton>>()
			.slots
			.insert(MouseButton::Right, SlotKey::Legs);

		app.update();

		app.world
			.resource_mut::<Input<MouseButton>>()
			.release(MouseButton::Right);

		app.update();

		let agent = app.world.entity(agent);
		let schedule = agent.get::<Schedule>();

		assert_eq!(None, schedule);
	}

	#[test]
	fn do_not_set_transition_when_previously_set_with_other_key() {
		let mut app = setup();
		let slots = Slots(
			[
				(
					SlotKey::Hand(Side::Off),
					Slot {
						entity: Entity::from_raw(42),
						item: Some(Item {
							skill: Some(Skill {
								name: "off skill",
								..default()
							}),
							..default()
						}),
						combo_skill: None,
					},
				),
				(
					SlotKey::Hand(Side::Main),
					Slot {
						entity: Entity::from_raw(44),
						item: Some(Item {
							skill: Some(Skill {
								name: "main skill",
								..default()
							}),
							..default()
						}),
						combo_skill: None,
					},
				),
			]
			.into(),
		);
		let agent = app.world.spawn((TestAgent, slots)).id();

		let mut mouse_map = app.world.resource_mut::<SlotMap<MouseButton>>();
		mouse_map
			.slots
			.insert(MouseButton::Left, SlotKey::Hand(Side::Off));
		mouse_map
			.slots
			.insert(MouseButton::Right, SlotKey::Hand(Side::Main));

		app.world
			.resource_mut::<Input<MouseButton>>()
			.press(MouseButton::Left);

		app.update();

		app.world
			.resource_mut::<Input<MouseButton>>()
			.clear_just_pressed(MouseButton::Left);

		app.world
			.resource_mut::<Input<MouseButton>>()
			.clear_just_pressed(MouseButton::Right);
		app.world
			.resource_mut::<NextState<MouseContext<MouseButton>>>()
			.set(MouseContext::JustReleased(MouseButton::Right));

		app.update();

		let agent = app.world.entity(agent);
		let schedule = agent.get::<Schedule>();

		assert_eq!(
			Some(&Schedule::Override((
				SlotKey::Hand(Side::Off),
				Skill {
					name: "off skill",
					..default()
				}
			))),
			schedule
		);
	}
}
