use crate::{
	components::{Schedule, Slot, SlotKey, Slots},
	resources::SlotMap,
};
use bevy::prelude::{Component, Input, KeyCode, MouseButton, Query, Res, With};

fn set_schedule<TBehavior>(
	slot: &mut Slot<TBehavior>,
	schedule: Schedule,
	triggered_slot_keys: &[&SlotKey],
	slot_key: &SlotKey,
) {
	if !triggered_slot_keys.contains(&slot_key) {
		return;
	}
	slot.schedule = Some(schedule);
}

fn set_schedules<TBehavior>(
	slots: &mut Slots<TBehavior>,
	schedule: Schedule,
	triggered_slot_keys: &[&SlotKey],
) {
	for (slot_key, slot) in slots.0.iter_mut() {
		set_schedule(slot, schedule, triggered_slot_keys, slot_key);
	}
}

pub fn schedule_slots_via_mouse<TAgent: Component, TBehavior: 'static>(
	mouse: Res<Input<MouseButton>>,
	keys: Res<Input<KeyCode>>,
	mouse_button_map: Res<SlotMap<MouseButton>>,
	mut query: Query<&mut Slots<TBehavior>, With<TAgent>>,
) {
	let schedule = match keys.pressed(KeyCode::ShiftLeft) {
		true => Schedule::Enqueue,
		false => Schedule::Override,
	};
	let triggered_slot_keys: Vec<&SlotKey> = mouse
		.get_just_pressed()
		.filter_map(|mouse_button| mouse_button_map.0.get(mouse_button))
		.collect();

	for mut slots in &mut query {
		set_schedules(&mut slots, schedule, &triggered_slot_keys);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{Schedule, Side, Slot, SlotKey, Slots},
		resources::SlotMap,
	};
	use bevy::prelude::{App, Component, Entity, Input, KeyCode, MouseButton, Update};

	#[derive(Component)]
	struct Agent;

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
		let mut app = setup();
		let slots = Slots(
			[(
				SlotKey::Legs,
				Slot::<MockBehavior>::new(Entity::from_raw(42), None),
			)]
			.into(),
		);
		let slots = app.world.spawn((Agent, slots)).id();

		app.world
			.resource_mut::<SlotMap<MouseButton>>()
			.0
			.insert(MouseButton::Right, SlotKey::Legs);
		app.world
			.resource_mut::<Input<MouseButton>>()
			.press(MouseButton::Right);

		app.update();

		let slots = app
			.world
			.entity(slots)
			.get::<Slots<MockBehavior>>()
			.unwrap();
		let slot = slots.0.get(&SlotKey::Legs).unwrap();

		assert_eq!(Some(Schedule::Override), slot.schedule);
	}

	#[test]
	fn do_not_set_when_not_on_agent() {
		let mut app = setup();
		let slots = Slots(
			[(
				SlotKey::Legs,
				Slot::<MockBehavior>::new(Entity::from_raw(42), None),
			)]
			.into(),
		);
		let slots = app.world.spawn(slots).id();

		app.world
			.resource_mut::<SlotMap<MouseButton>>()
			.0
			.insert(MouseButton::Right, SlotKey::Legs);
		app.world
			.resource_mut::<Input<MouseButton>>()
			.press(MouseButton::Right);

		app.update();

		let slots = app
			.world
			.entity(slots)
			.get::<Slots<MockBehavior>>()
			.unwrap();
		let slot = slots.0.get(&SlotKey::Legs).unwrap();

		assert_eq!(None, slot.schedule);
	}

	#[test]
	fn do_not_set_when_mouse_button_not_correctly_mapped() {
		let mut app = setup();
		let slots = Slots(
			[(
				SlotKey::Legs,
				Slot::<MockBehavior>::new(Entity::from_raw(42), None),
			)]
			.into(),
		);
		let slots = app.world.spawn((Agent, slots)).id();

		app.world
			.resource_mut::<SlotMap<MouseButton>>()
			.0
			.insert(MouseButton::Right, SlotKey::Hand(Side::Left));
		app.world
			.resource_mut::<Input<MouseButton>>()
			.press(MouseButton::Right);

		app.update();

		let slots = app
			.world
			.entity(slots)
			.get::<Slots<MockBehavior>>()
			.unwrap();
		let slot = slots.0.get(&SlotKey::Legs).unwrap();

		assert_eq!(None, slot.schedule);
	}

	#[test]
	fn set_enqueue() {
		let mut app = setup();
		let slots = Slots(
			[(
				SlotKey::Legs,
				Slot::<MockBehavior>::new(Entity::from_raw(42), None),
			)]
			.into(),
		);
		let slots = app.world.spawn((Agent, slots)).id();

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

		let slots = app
			.world
			.entity(slots)
			.get::<Slots<MockBehavior>>()
			.unwrap();
		let slot = slots.0.get(&SlotKey::Legs).unwrap();

		assert_eq!(Some(Schedule::Enqueue), slot.schedule);
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
				Slot::<MockBehavior>::new(Entity::from_raw(42), None),
			)]
			.into(),
		);
		let slots_entity = app.world.spawn((Agent, slots)).id();
		app.world
			.resource_mut::<Input<MouseButton>>()
			.clear_just_pressed(MouseButton::Right);

		app.update();

		let slots = app
			.world
			.entity(slots_entity)
			.get::<Slots<MockBehavior>>()
			.unwrap();
		let slot = slots.0.get(&SlotKey::Legs).unwrap();

		assert_eq!(None, slot.schedule);
	}
}
