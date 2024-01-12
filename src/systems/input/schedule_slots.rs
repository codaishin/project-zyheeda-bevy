use crate::{
	components::{Schedule, ScheduleMode, SlotKey, Slots},
	resources::SlotMap,
	skill::Skill,
	states::MouseContext,
};
use bevy::{
	ecs::schedule::State,
	prelude::{Commands, Component, Entity, Input, KeyCode, Query, Res, With},
};
use std::{fmt::Debug, hash::Hash};

pub fn schedule_slots<TKey: Copy + Eq + Hash + Debug + Send + Sync, TAgent: Component>(
	input: Res<Input<TKey>>,
	mouse_context: Option<Res<State<MouseContext<TKey>>>>,
	keys: Res<Input<KeyCode>>,
	slot_map: Res<SlotMap<TKey>>,
	agents: Query<(Entity, &Slots), With<TAgent>>,
	mut commands: Commands,
) {
	let mut triggered_slot_keys: Vec<_> = input
		.get_just_pressed()
		.filter_map(|key| slot_map.slots.get(key))
		.cloned()
		.collect();

	if let Some(slot_key) = triggered_mouse_context_key(mouse_context, &slot_map) {
		triggered_slot_keys.push(slot_key);
	}

	if triggered_slot_keys.is_empty() {
		return;
	}

	let mode = match keys.pressed(KeyCode::ShiftLeft) {
		true => ScheduleMode::Enqueue,
		false => ScheduleMode::Override,
	};

	for (agent, slots) in &agents {
		let behaviors = filter_triggered_behaviors(slots, &triggered_slot_keys);
		if !behaviors.is_empty() {
			commands.entity(agent).insert(Schedule {
				mode,
				skills: behaviors.into_iter().collect(),
			});
		}
	}
}

fn triggered_mouse_context_key<TKey: Copy + Eq + Hash + Debug + Send + Sync>(
	mouse_context: Option<Res<State<MouseContext<TKey>>>>,
	slot_map: &Res<SlotMap<TKey>>,
) -> Option<SlotKey> {
	match *mouse_context?.get() {
		MouseContext::Triggered(key) => slot_map.slots.get(&key).copied(),
		_ => None,
	}
}

fn filter_triggered_behaviors(
	slots: &Slots,
	triggered_slot_keys: &[SlotKey],
) -> Vec<(SlotKey, Skill)> {
	slots
		.0
		.iter()
		.filter(|(slot_key, ..)| triggered_slot_keys.contains(slot_key))
		.filter_map(|(key, slot)| {
			Some((*key, slot.combo_skill.clone().or(slot.item.clone()?.skill)?))
		})
		.collect()
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{Item, Schedule, ScheduleMode, Side, Slot, SlotKey, Slots},
		resources::SlotMap,
	};
	use bevy::{
		ecs::schedule::NextState,
		prelude::{default, App, Component, Entity, Input, KeyCode, MouseButton, Update},
	};

	#[derive(Component)]
	struct TestAgent;

	fn setup() -> App {
		let mut app = App::new();

		app.insert_resource(Input::<MouseButton>::default())
			.insert_resource(Input::<KeyCode>::default())
			.insert_resource(SlotMap::<MouseButton>::new([]))
			.add_state::<MouseContext<MouseButton>>()
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
			Some(&Schedule {
				mode: ScheduleMode::Override,
				skills: [(
					SlotKey::Legs,
					Skill {
						name: "skill",
						..default()
					}
				)]
				.into()
			}),
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
			Some(&Schedule {
				mode: ScheduleMode::Override,
				skills: [(
					SlotKey::Legs,
					Skill {
						name: "alternative skill",
						..default()
					}
				)]
				.into()
			}),
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
			Some(&Schedule {
				mode: ScheduleMode::Enqueue,
				skills: [(
					SlotKey::Hand(Side::Main),
					Skill {
						name: "skill",
						..default()
					}
				)]
				.into()
			}),
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
			Some(&Schedule {
				mode: ScheduleMode::Enqueue,
				skills: [(
					SlotKey::Hand(Side::Main),
					Skill {
						name: "alternative skill",
						..default()
					}
				)]
				.into()
			}),
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
			.set(MouseContext::Triggered(MouseButton::Right));

		app.update();

		let agent = app.world.entity(agent);
		let schedule = agent.get::<Schedule>();

		assert_eq!(
			Some(&Schedule {
				mode: ScheduleMode::Override,
				skills: [(
					SlotKey::Legs,
					Skill {
						name: "skill",
						..default()
					}
				)]
				.into()
			}),
			schedule
		);
	}
}
