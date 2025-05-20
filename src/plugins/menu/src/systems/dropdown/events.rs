use crate::{components::dropdown::Dropdown, events::DropdownEvent};
use bevy::prelude::{Added, Entity, EventWriter, Query, RemovedComponents};

pub(crate) fn dropdown_events<TItem: Send + Sync + 'static>(
	dropdowns: Query<Entity, Added<Dropdown<TItem>>>,
	mut removed_dropdowns: RemovedComponents<Dropdown<TItem>>,
	mut events: EventWriter<DropdownEvent>,
) {
	for entity in &dropdowns {
		events.write(DropdownEvent::Added(entity));
	}

	for entity in removed_dropdowns.read() {
		events.write(DropdownEvent::Removed(entity));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		prelude::Events,
	};
	use common::test_tools::utils::SingleThreadedApp;

	struct _Item;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_event::<DropdownEvent>();
		app.add_systems(Update, dropdown_events::<_Item>);

		app
	}

	#[test]
	fn send_added_event() {
		let mut app = setup();
		let dropdown = app.world_mut().spawn(Dropdown::<_Item>::default()).id();

		app.update();

		let events = app.world().resource::<Events<DropdownEvent>>();
		let mut cursor = events.get_cursor();

		assert_eq!(
			vec![&DropdownEvent::Added(dropdown)],
			cursor.read(events).collect::<Vec<_>>()
		)
	}

	#[test]
	fn do_not_send_added_event_when_no_dropdown_present() {
		let mut app = setup();
		app.world_mut().spawn_empty();

		app.update();

		let events = app.world().resource::<Events<DropdownEvent>>();
		let mut cursor = events.get_cursor();

		assert_eq!(
			vec![] as Vec<&DropdownEvent>,
			cursor.read(events).collect::<Vec<_>>()
		)
	}

	#[test]
	fn send_added_event_only_when_added() {
		let mut app = setup();
		let dropdown = app.world_mut().spawn(Dropdown::<_Item>::default()).id();

		app.update();
		app.update();

		let events = app.world().resource::<Events<DropdownEvent>>();
		let mut cursor = events.get_cursor();

		assert_eq!(
			vec![&DropdownEvent::Added(dropdown)],
			cursor.read(events).collect::<Vec<_>>()
		)
	}

	#[test]
	fn send_remove_event() {
		let mut app = setup();
		let dropdown = app.world_mut().spawn(Dropdown::<_Item>::default()).id();

		app.update();

		app.world_mut()
			.entity_mut(dropdown)
			.remove::<Dropdown<_Item>>();

		app.update();

		let events = app.world().resource::<Events<DropdownEvent>>();
		let mut cursor = events.get_cursor();

		assert_eq!(
			vec![
				&DropdownEvent::Added(dropdown),
				&DropdownEvent::Removed(dropdown)
			],
			cursor.read(events).collect::<Vec<_>>()
		)
	}
}
