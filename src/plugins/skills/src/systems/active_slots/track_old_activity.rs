use crate::components::held_slots::{HeldSlots, Old};
use bevy::prelude::*;

impl HeldSlots<Old> {
	pub(crate) fn update_from<TFrame>(
		mut active_slots: Query<(&mut Self, &HeldSlots<TFrame>), Changed<HeldSlots<TFrame>>>,
	) where
		TFrame: 'static,
	{
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

	struct _Source;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			(
				HeldSlots::<Old>::update_from::<_Source>,
				IsChanged::<HeldSlots<Old>>::detect,
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
				HeldSlots::<_Source>::from([SlotKey(1), SlotKey(2)]),
				HeldSlots::<Old>::default(),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&HeldSlots::from([SlotKey(1), SlotKey(2)])),
			app.world().entity(entity).get::<HeldSlots<Old>>(),
		);
	}

	#[test]
	fn override_old_active() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				HeldSlots::<_Source>::from([SlotKey(1), SlotKey(2)]),
				HeldSlots::<Old>::from([SlotKey(11), SlotKey(12)]),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&HeldSlots::from([SlotKey(1), SlotKey(2)])),
			app.world().entity(entity).get::<HeldSlots<Old>>(),
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				HeldSlots::<_Source>::from([SlotKey(1), SlotKey(2)]),
				HeldSlots::<Old>::default(),
			))
			.id();

		app.update();
		app.update();

		assert_eq!(
			Some(&IsChanged::FALSE),
			app.world()
				.entity(entity)
				.get::<IsChanged<HeldSlots<Old>>>(),
		);
	}

	#[test]
	fn act_again_if_current_active_slots_changed() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				HeldSlots::<_Source>::from([SlotKey(1), SlotKey(2)]),
				HeldSlots::<Old>::default(),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<HeldSlots<_Source>>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some(&IsChanged::TRUE),
			app.world()
				.entity(entity)
				.get::<IsChanged<HeldSlots<Old>>>(),
		);
	}
}
