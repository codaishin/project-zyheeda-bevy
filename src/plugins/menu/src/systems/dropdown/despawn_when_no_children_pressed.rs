use crate::components::dropdown::DropdownUI;
use bevy::prelude::*;
use common::tools::Focus;

pub(crate) fn dropdown_despawn_when_no_children_pressed<TItem: Sync + Send + 'static>(
	focus: In<Focus>,
	commands: Commands,
	dropdowns: Query<(Entity, &DropdownUI<TItem>)>,
	interactions: Query<&Interaction>,
) -> Focus {
	match focus.0 {
		Focus::New(new_focus) => despawn_and_unfocus(dropdowns, interactions, commands, new_focus),
		Focus::Unchanged => Focus::Unchanged,
	}
}

fn despawn_and_unfocus<TItem: Sync + Send + 'static>(
	dropdown_uis: Query<(Entity, &DropdownUI<TItem>)>,
	interactions: Query<&Interaction>,
	mut commands: Commands,
	mut new_focus: Vec<Entity>,
) -> Focus {
	let has_no_pressed_child_dropdown = |(_, dropdown_ui): &(Entity, &DropdownUI<TItem>)| {
		!dropdown_ui
			.child_dropdowns
			.iter()
			.filter_map(|entity| interactions.get(*entity).ok())
			.any(|interaction| interaction == &Interaction::Pressed)
	};

	for (entity, dropdown_ui) in dropdown_uis.iter().filter(has_no_pressed_child_dropdown) {
		despawn_entity(&mut commands, entity);
		unfocus(&mut new_focus, &dropdown_ui.source);
	}

	Focus::New(new_focus)
}

fn despawn_entity(commands: &mut Commands, entity: Entity) {
	let Ok(mut entity) = commands.get_entity(entity) else {
		return;
	};
	entity.despawn();
}

fn unfocus(new_focus: &mut Vec<Entity>, despawned: &Entity) {
	new_focus.retain(|focused| focused != despawned);
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Debug, PartialEq)]
	struct _Item;

	#[derive(Resource, Default)]
	struct _In(pub Focus);

	#[derive(Resource, Debug, PartialEq)]
	struct _Result(Focus);

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<_In>();
		app.add_systems(
			Update,
			(|new_active: Res<_In>| new_active.0.clone())
				.pipe(dropdown_despawn_when_no_children_pressed::<_Item>)
				.pipe(|focus: In<Focus>, mut commands: Commands| {
					commands.insert_resource(_Result(focus.0))
				}),
		);

		app
	}

	#[test]
	fn despawn_dropdown_ui() {
		let mut app = setup();
		app.world_mut()
			.spawn(DropdownUI::<_Item>::new(Entity::from_raw(42)));

		app.world_mut().insert_resource(_In(Focus::New(vec![])));

		app.update();

		let dropdown_uis = app
			.world()
			.iter_entities()
			.find(|e| e.contains::<DropdownUI<_Item>>());

		assert!(dropdown_uis.is_none());
	}

	#[test]
	fn do_not_despawn_non_dropdown_ui_entities() {
		#[derive(Component)]
		struct _Other;

		let mut app = setup();
		app.world_mut()
			.spawn(DropdownUI::<_Item>::new(Entity::from_raw(42)));
		app.world_mut().spawn(_Other);

		app.world_mut().insert_resource(_In(Focus::New(vec![])));

		app.update();

		let other = app.world().iter_entities().find(|e| e.contains::<_Other>());

		assert!(other.is_some());
	}

	#[test]
	fn despawn_dropdown_ui_recursively() {
		#[derive(Component)]
		struct _Child;

		let mut app = setup();
		app.world_mut()
			.spawn(DropdownUI::<_Item>::new(Entity::from_raw(42)))
			.with_children(|dropdown_ui| {
				dropdown_ui.spawn(_Child);
			});

		app.world_mut().insert_resource(_In(Focus::New(vec![])));

		app.update();

		let children = app.world().iter_entities().find(|e| e.contains::<_Child>());

		assert!(children.is_none());
	}

	#[test]
	fn do_nothing_when_focus_unchanged() {
		let mut app = setup();
		app.world_mut()
			.spawn(DropdownUI::<_Item>::new(Entity::from_raw(42)));

		app.world_mut().insert_resource(_In(Focus::Unchanged));

		app.update();

		let dropdown_uis = app
			.world()
			.iter_entities()
			.find(|e| e.contains::<DropdownUI<_Item>>());

		assert!(dropdown_uis.is_some());
	}

	#[test]
	fn return_new_focus() {
		let mut app = setup();
		let focus = Focus::New(vec![
			Entity::from_raw(42),
			Entity::from_raw(11),
			Entity::from_raw(69),
		]);

		app.world_mut().insert_resource(_In(focus.clone()));

		app.update();

		assert_eq!(&_Result(focus), app.world().resource::<_Result>());
	}

	#[test]
	fn return_unchanged_focus() {
		let mut app = setup();
		let focus = Focus::Unchanged;

		app.world_mut().insert_resource(_In(focus.clone()));

		app.update();

		assert_eq!(&_Result(focus), app.world().resource::<_Result>());
	}

	#[test]
	fn return_new_focus_without_source_of_despawned() {
		let mut app = setup();
		let source = Entity::from_raw(101);

		app.world_mut().spawn(DropdownUI::<_Item>::new(source));
		app.world_mut().insert_resource(_In(Focus::New(vec![
			Entity::from_raw(42),
			source,
			Entity::from_raw(69),
			Entity::from_raw(77),
		])));

		app.update();

		assert_eq!(
			&_Result(Focus::New(vec![
				Entity::from_raw(42),
				Entity::from_raw(69),
				Entity::from_raw(77),
			])),
			app.world().resource::<_Result>()
		);
	}

	#[test]
	fn do_not_despawn_dropdown_ui_when_children_pressed() {
		let mut app = setup();

		let source = Entity::from_raw(42);
		let child = app.world_mut().spawn(Interaction::Pressed).id();
		let dropdown = app
			.world_mut()
			.spawn(DropdownUI::<_Item>::new(source).with_child_dropdowns([child]))
			.id();

		app.world_mut().insert_resource(_In(Focus::New(vec![])));

		app.update();

		let dropdown = app.world().entity(dropdown);

		assert_eq!(
			Some(&DropdownUI::<_Item>::new(source).with_child_dropdowns([child])),
			dropdown.get::<DropdownUI<_Item>>()
		);
	}
}
