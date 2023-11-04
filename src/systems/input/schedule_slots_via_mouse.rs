use crate::{
	components::{GetBehaviorFn, Schedule, ScheduleMode, SlotKey, Slots},
	resources::SlotMap,
};
use bevy::prelude::{Commands, Component, Entity, Input, KeyCode, MouseButton, Query, Res, With};

fn filter_triggered_behavior_fns<TBehavior>(
	slots: &Slots<TBehavior>,
	triggered_slot_keys: &[&SlotKey],
) -> Vec<GetBehaviorFn<TBehavior>> {
	slots
		.0
		.iter()
		.filter(|(slot_key, ..)| triggered_slot_keys.contains(slot_key))
		.filter_map(|(_, slot)| slot.get_behavior)
		.collect()
}

pub fn schedule_slots_via_mouse<TAgent: Component, TBehavior: Sync + Send + 'static>(
	mouse: Res<Input<MouseButton>>,
	keys: Res<Input<KeyCode>>,
	mouse_button_map: Res<SlotMap<MouseButton>>,
	agents: Query<(Entity, &Slots<TBehavior>), With<TAgent>>,
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
		let get_behaviors = filter_triggered_behavior_fns(slots, &triggered_slot_keys);
		if !get_behaviors.is_empty() {
			commands.entity(agent).insert(Schedule::<TBehavior> {
				mode,
				get_behaviors,
			});
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{Schedule, ScheduleMode, Side, Slot, SlotKey, Slots},
		resources::SlotMap,
	};
	use bevy::prelude::{App, Component, Entity, Input, KeyCode, MouseButton, Ray, Update};

	#[derive(Component)]
	struct Agent;

	#[derive(PartialEq, Debug)]
	struct MockBehavior;

	fn setup() -> App {
		let mut app = App::new();
		let mouse = Input::<MouseButton>::default();
		let keys = Input::<KeyCode>::default();
		let mouse_settings = SlotMap::<MouseButton>::new([]);

		app.insert_resource(mouse);
		app.insert_resource(keys);
		app.insert_resource(mouse_settings);
		app.add_systems(Update, schedule_slots_via_mouse::<Agent, MockBehavior>);
		app
	}

	#[test]
	fn set_override() {
		fn get_behavior(_ray: Ray) -> Option<MockBehavior> {
			None
		}

		let mut app = setup();
		let slots = Slots(
			[(
				SlotKey::Legs,
				Slot {
					entity: Entity::from_raw(42),
					get_behavior: Some(get_behavior),
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
		let schedule = agent.get::<Schedule<MockBehavior>>();

		assert_eq!(
			Some(&Schedule::<MockBehavior> {
				mode: ScheduleMode::Override,
				get_behaviors: vec![get_behavior]
			}),
			schedule
		);
	}

	#[test]
	fn do_not_set_when_not_on_agent() {
		fn get_behavior(_ray: Ray) -> Option<MockBehavior> {
			None
		}

		let mut app = setup();
		let slots = Slots(
			[(
				SlotKey::Legs,
				Slot {
					entity: Entity::from_raw(42),
					get_behavior: Some(get_behavior),
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
		let schedule = agent.get::<Schedule<MockBehavior>>();

		assert_eq!(None, schedule);
	}

	#[test]
	fn do_not_set_when_mouse_button_not_correctly_mapped() {
		fn get_behavior(_ray: Ray) -> Option<MockBehavior> {
			None
		}

		let mut app = setup();
		let slots = Slots(
			[(
				SlotKey::Legs,
				Slot {
					entity: Entity::from_raw(42),
					get_behavior: Some(get_behavior),
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
		let schedule = agent.get::<Schedule<MockBehavior>>();

		assert_eq!(None, schedule);
	}

	#[test]
	fn set_enqueue() {
		fn get_behavior(_ray: Ray) -> Option<MockBehavior> {
			None
		}

		let mut app = setup();
		let slots = Slots(
			[(
				SlotKey::Legs,
				Slot {
					entity: Entity::from_raw(42),
					get_behavior: Some(get_behavior),
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
		app.world
			.resource_mut::<Input<KeyCode>>()
			.press(KeyCode::ShiftLeft);

		app.update();

		let agent = app.world.entity(agent);
		let schedule = agent.get::<Schedule<MockBehavior>>();

		assert_eq!(
			Some(&Schedule::<MockBehavior> {
				mode: ScheduleMode::Enqueue,
				get_behaviors: vec![get_behavior]
			}),
			schedule
		);
	}

	#[test]
	fn do_not_set_when_not_just_mouse_pressed() {
		fn get_behavior(_ray: Ray) -> Option<MockBehavior> {
			None
		}

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
					get_behavior: Some(get_behavior),
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
		let schedule = agent.get::<Schedule<MockBehavior>>();

		assert_eq!(None, schedule);
	}
}
