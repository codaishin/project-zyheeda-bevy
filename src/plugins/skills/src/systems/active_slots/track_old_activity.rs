use crate::components::active_slots::{ActiveSlots, Current, Old};
use bevy::prelude::*;

impl ActiveSlots<Old> {
	pub(crate) fn track(
		mut active_slots: Query<(&mut Self, &ActiveSlots<Current>), Changed<ActiveSlots<Current>>>,
	) {
		for (mut old, current) in &mut active_slots {
			old.slots = current.slots.clone();
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::tools::action_key::slot::SlotKey;
	use testing::{IsChanged, SingleThreadedApp};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			(
				ActiveSlots::<Old>::track,
				IsChanged::<ActiveSlots<Old>>::detect,
			)
				.chain(),
		);

		app
	}

	#[test]
	fn clone_currently_active_into_old_active() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				ActiveSlots::<Current>::from([SlotKey(1), SlotKey(2)]),
				ActiveSlots::<Old>::default(),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&ActiveSlots::from([SlotKey(1), SlotKey(2)])),
			app.world().entity(entity).get::<ActiveSlots<Old>>(),
		);
	}

	#[test]
	fn override_old_active() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				ActiveSlots::<Current>::from([SlotKey(1), SlotKey(2)]),
				ActiveSlots::<Old>::from([SlotKey(11), SlotKey(12)]),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&ActiveSlots::from([SlotKey(1), SlotKey(2)])),
			app.world().entity(entity).get::<ActiveSlots<Old>>(),
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				ActiveSlots::<Current>::from([SlotKey(1), SlotKey(2)]),
				ActiveSlots::<Old>::default(),
			))
			.id();

		app.update();
		app.update();

		assert_eq!(
			Some(&IsChanged::FALSE),
			app.world()
				.entity(entity)
				.get::<IsChanged<ActiveSlots<Old>>>(),
		);
	}

	#[test]
	fn act_again_if_current_active_slots_changed() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				ActiveSlots::<Current>::from([SlotKey(1), SlotKey(2)]),
				ActiveSlots::<Old>::default(),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<ActiveSlots<Current>>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some(&IsChanged::TRUE),
			app.world()
				.entity(entity)
				.get::<IsChanged<ActiveSlots<Old>>>(),
		);
	}
}
