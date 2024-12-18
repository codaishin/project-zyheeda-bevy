use crate::{components::dropdown::DropdownUI, events::DropdownEvent};
use bevy::prelude::*;

pub(crate) fn dropdown_track_child_dropdowns<TItem: Send + Sync + 'static>(
	mut events: EventReader<DropdownEvent>,
	mut dropdown_uis: Query<&mut DropdownUI<TItem>>,
	parents: Query<&Parent>,
) {
	for event in events.read() {
		match event {
			DropdownEvent::Added(child) => add_child(&mut dropdown_uis, &parents, child),
			DropdownEvent::Removed(child) => remove_child(&mut dropdown_uis, child),
		}
	}
}

fn add_child<TItem: Send + Sync + 'static>(
	dropdown_uis: &mut Query<&mut DropdownUI<TItem>>,
	parents: &Query<&Parent>,
	dropdown_entity: &Entity,
) {
	for ancestor in parents.iter_ancestors(*dropdown_entity) {
		let Ok(mut dropdown_ui) = dropdown_uis.get_mut(ancestor) else {
			continue;
		};
		dropdown_ui.child_dropdowns.insert(*dropdown_entity);
	}
}

fn remove_child<TItem: Send + Sync + 'static>(
	dropdown_uis: &mut Query<&mut DropdownUI<TItem>>,
	dropdown_entity: &Entity,
) {
	for mut dropdown_ui in dropdown_uis {
		dropdown_ui.child_dropdowns.remove(dropdown_entity);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Debug, PartialEq)]
	struct _Item;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_event::<DropdownEvent>();
		app.add_systems(Update, dropdown_track_child_dropdowns::<_Item>);

		app
	}

	#[test]
	fn add_child_entity() {
		let mut app = setup();

		let source = Entity::from_raw(42);
		let dropdown = app.world_mut().spawn(DropdownUI::<_Item>::new(source)).id();
		let child_dropdown = app.world_mut().spawn_empty().set_parent(dropdown).id();
		app.world_mut()
			.send_event(DropdownEvent::Added(child_dropdown));

		app.update();

		let dropdown = app.world().entity(dropdown);

		assert_eq!(
			Some(&DropdownUI::new(source).with_child_dropdowns([child_dropdown])),
			dropdown.get::<DropdownUI<_Item>>()
		)
	}

	#[test]
	fn do_not_add_child_entity_when_not_child_of_dropdown_ui() {
		let mut app = setup();

		let source = Entity::from_raw(42);
		let dropdown = app.world_mut().spawn(DropdownUI::<_Item>::new(source)).id();
		let child_dropdown = app.world_mut().spawn_empty().id();
		app.world_mut()
			.send_event(DropdownEvent::Added(child_dropdown));

		app.update();

		let dropdown = app.world().entity(dropdown);

		assert_eq!(
			Some(&DropdownUI::new(source).with_child_dropdowns([])),
			dropdown.get::<DropdownUI<_Item>>()
		)
	}

	#[test]
	fn do_not_add_child_entity_when_dropdown_ui_has_children_but_event_entity_is_not_one_of_them() {
		let mut app = setup();

		let source = Entity::from_raw(42);
		let dropdown = app.world_mut().spawn(DropdownUI::<_Item>::new(source)).id();
		app.world_mut().spawn_empty().set_parent(dropdown);
		let child_dropdown = app.world_mut().spawn_empty().id();
		app.world_mut()
			.send_event(DropdownEvent::Added(child_dropdown));

		app.update();

		let dropdown = app.world().entity(dropdown);

		assert_eq!(
			Some(&DropdownUI::new(source).with_child_dropdowns([])),
			dropdown.get::<DropdownUI<_Item>>()
		)
	}

	#[test]
	fn add_child_entity_when_nested_deeply_under_dropdown_ui() {
		let mut app = setup();

		let source = Entity::from_raw(42);
		let dropdown = app.world_mut().spawn(DropdownUI::<_Item>::new(source)).id();
		let in_between = app.world_mut().spawn_empty().set_parent(dropdown).id();
		let child_dropdown = app.world_mut().spawn_empty().set_parent(in_between).id();
		app.world_mut()
			.send_event(DropdownEvent::Added(child_dropdown));

		app.update();

		let dropdown = app.world().entity(dropdown);

		assert_eq!(
			Some(&DropdownUI::new(source).with_child_dropdowns([child_dropdown])),
			dropdown.get::<DropdownUI<_Item>>()
		)
	}

	#[test]
	fn add_multiple_child_entities() {
		let mut app = setup();

		let source = Entity::from_raw(42);
		let dropdown = app.world_mut().spawn(DropdownUI::<_Item>::new(source)).id();
		let child_dropdown_a = app.world_mut().spawn_empty().set_parent(dropdown).id();
		let child_dropdown_b = app.world_mut().spawn_empty().set_parent(dropdown).id();
		app.world_mut()
			.send_event(DropdownEvent::Added(child_dropdown_a));
		app.world_mut()
			.send_event(DropdownEvent::Added(child_dropdown_b));

		app.update();

		let dropdown = app.world().entity(dropdown);

		assert_eq!(
			Some(
				&DropdownUI::new(source).with_child_dropdowns([child_dropdown_a, child_dropdown_b])
			),
			dropdown.get::<DropdownUI<_Item>>()
		)
	}

	#[test]
	fn remove_child_unchecked() {
		let mut app = setup();

		let source = Entity::from_raw(42);
		let dropdown = app
			.world_mut()
			.spawn(
				DropdownUI::<_Item>::new(source)
					.with_child_dropdowns([Entity::from_raw(1), Entity::from_raw(2)]),
			)
			.id();
		app.world_mut()
			.send_event(DropdownEvent::Removed(Entity::from_raw(2)));

		app.update();

		let dropdown = app.world().entity(dropdown);

		assert_eq!(
			Some(&DropdownUI::new(source).with_child_dropdowns([Entity::from_raw(1)])),
			dropdown.get::<DropdownUI<_Item>>()
		)
	}
}
