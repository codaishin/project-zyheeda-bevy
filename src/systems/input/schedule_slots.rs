use crate::{
	behaviors::Behavior,
	components::{Schedule, ScheduleMode, SlotKey, Slots},
	resources::SlotMap,
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
				behaviors: behaviors.into_iter().collect(),
			});
		}
	}
}

fn filter_triggered_behaviors(
	slots: &Slots,
	triggered_slot_keys: &[&SlotKey],
) -> Vec<(SlotKey, Behavior)> {
	slots
		.0
		.iter()
		.filter(|(slot_key, ..)| triggered_slot_keys.contains(slot_key))
		.filter_map(|(key, slot)| slot.behavior.map(|b| (*key, b)))
		.collect()
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{Schedule, ScheduleMode, Side, Slot, SlotKey, Slots},
		resources::SlotMap,
	};
	use bevy::{
		ecs::system::EntityCommands,
		prelude::{App, Component, Entity, Input, KeyCode, MouseButton, Ray, Update},
	};

	#[derive(Component)]
	struct Agent;

	fn setup() -> App {
		let mut app = App::new();
		let mouse = Input::<MouseButton>::default();
		let keys = Input::<KeyCode>::default();
		let mouse_settings = SlotMap::<MouseButton>::new([]);

		app.insert_resource(mouse);
		app.insert_resource(keys);
		app.insert_resource(mouse_settings);
		app.add_systems(Update, schedule_slots::<MouseButton, Agent>);
		app
	}

	fn fake_behavior_insert<const T: char>(_entity: &mut EntityCommands, _ray: Ray) {}

	#[test]
	fn set_override() {
		let mut app = setup();
		let slots = Slots(
			[(
				SlotKey::Legs,
				Slot {
					entity: Entity::from_raw(42),
					behavior: Some(Behavior {
						insert_fn: fake_behavior_insert::<'!'>,
					}),
				},
			)]
			.into(),
		);
		let agent = app.world.spawn((Agent, slots)).id();

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
				behaviors: [(
					SlotKey::Legs,
					Behavior {
						insert_fn: fake_behavior_insert::<'!'>,
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
					behavior: Some(Behavior {
						insert_fn: fake_behavior_insert::<'!'>,
					}),
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
					behavior: Some(Behavior {
						insert_fn: fake_behavior_insert::<'!'>,
					}),
				},
			)]
			.into(),
		);
		let agent = app.world.spawn((Agent, slots)).id();

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
					behavior: Some(Behavior {
						insert_fn: fake_behavior_insert::<'/'>,
					}),
				},
			)]
			.into(),
		);
		let agent = app.world.spawn((Agent, slots)).id();

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
				behaviors: [(
					SlotKey::Hand(Side::Right),
					Behavior {
						insert_fn: fake_behavior_insert::<'/'>,
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
					behavior: Some(Behavior {
						insert_fn: fake_behavior_insert::<'!'>,
					}),
				},
			)]
			.into(),
		);
		let agent = app.world.spawn((Agent, slots)).id();
		app.world
			.resource_mut::<Input<MouseButton>>()
			.clear_just_pressed(MouseButton::Right);

		app.update();

		let agent = app.world.entity(agent);
		let schedule = agent.get::<Schedule>();

		assert_eq!(None, schedule);
	}
}
