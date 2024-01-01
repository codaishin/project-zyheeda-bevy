use crate::{
	components::{Schedule, ScheduleMode, SlotKey, Slots},
	resources::SlotMap,
	skill::Skill,
};
use bevy::prelude::{Commands, Component, Entity, Input, KeyCode, Query, Res, With};
use std::hash::Hash;

pub fn schedule_slots<TKey: Copy + Eq + Hash + Send + Sync, TAgent: Component>(
	mouse: Res<Input<TKey>>,
	keys: Res<Input<KeyCode>>,
	mouse_button_map: Res<SlotMap<TKey>>,
	agents: Query<(Entity, &Slots), With<TAgent>>,
	mut commands: Commands,
) {
	let triggered_slot_keys = mouse
		.get_just_pressed()
		.filter_map(|mouse_button| mouse_button_map.0.get(mouse_button))
		.collect::<Vec<&SlotKey>>();

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

fn filter_triggered_behaviors(
	slots: &Slots,
	triggered_slot_keys: &[&SlotKey],
) -> Vec<(SlotKey, Skill)> {
	slots
		.0
		.iter()
		.filter(|(slot_key, ..)| triggered_slot_keys.contains(slot_key))
		.filter_map(|(key, slot)| Some((*key, slot.alternative_skill.or(slot.item?.skill)?)))
		.collect()
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{Item, Schedule, ScheduleMode, Side, Slot, SlotKey, Slots},
		resources::SlotMap,
	};
	use bevy::prelude::{default, App, Component, Entity, Input, KeyCode, MouseButton, Update};

	#[derive(Component)]
	struct TestAgent;

	fn setup() -> App {
		let mut app = App::new();
		let mouse = Input::<MouseButton>::default();
		let keys = Input::<KeyCode>::default();
		let mouse_settings = SlotMap::<MouseButton>::new([]);

		app.insert_resource(mouse);
		app.insert_resource(keys);
		app.insert_resource(mouse_settings);
		app.add_systems(Update, schedule_slots::<MouseButton, TestAgent>);
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
					alternative_skill: None,
				},
			)]
			.into(),
		);
		let agent = app.world.spawn((TestAgent, slots)).id();

		app.world
			.resource_mut::<SlotMap<MouseButton>>()
			.0
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
					alternative_skill: Some(Skill {
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
			.0
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
					alternative_skill: None,
				},
			)]
			.into(),
		);
		let agent = app.world.spawn(slots).id();

		app.world
			.resource_mut::<SlotMap<MouseButton>>()
			.0
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
					alternative_skill: None,
				},
			)]
			.into(),
		);
		let agent = app.world.spawn((TestAgent, slots)).id();

		app.world
			.resource_mut::<SlotMap<MouseButton>>()
			.0
			.insert(MouseButton::Right, SlotKey::Hand(Side::Left));
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
				SlotKey::Hand(Side::Right),
				Slot {
					entity: Entity::from_raw(42),
					item: Some(Item {
						skill: Some(Skill {
							name: "skill",
							..default()
						}),
						..default()
					}),
					alternative_skill: None,
				},
			)]
			.into(),
		);
		let agent = app.world.spawn((TestAgent, slots)).id();

		app.world
			.resource_mut::<SlotMap<MouseButton>>()
			.0
			.insert(MouseButton::Left, SlotKey::Hand(Side::Right));
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
					SlotKey::Hand(Side::Right),
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
				SlotKey::Hand(Side::Right),
				Slot {
					entity: Entity::from_raw(42),
					item: Some(Item {
						skill: Some(Skill {
							name: "skill",
							..default()
						}),
						..default()
					}),
					alternative_skill: Some(Skill {
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
			.0
			.insert(MouseButton::Left, SlotKey::Hand(Side::Right));
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
					SlotKey::Hand(Side::Right),
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
			.0
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
					alternative_skill: None,
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
}
