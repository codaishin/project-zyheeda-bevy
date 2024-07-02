use super::spawn_focused::DropdownUI;
use crate::components::dropdown::Dropdown;
use bevy::{
	hierarchy::{DespawnRecursiveExt, Parent},
	prelude::{Commands, Entity, In, Query, With},
};
use common::tools::Focus;

pub(crate) fn dropdown_despawn_unfocused(
	focus: In<Focus>,
	mut commands: Commands,
	dropdowns: Query<(Entity, &Parent), With<Dropdown>>,
	dropdown_uis: Query<(Entity, &Parent), With<DropdownUI>>,
) {
	let Focus::New(new_focus) = focus.0 else {
		return;
	};
	let containers_with_active_dropdowns = dropdowns
		.iter()
		.filter_map(|(entity, container)| {
			if new_focus.contains(&entity) {
				Some(container.get())
			} else {
				None
			}
		})
		.collect::<Vec<_>>();

	for (entity, container) in &dropdown_uis {
		if containers_with_active_dropdowns.contains(&container.get()) {
			continue;
		}
		let Some(entity) = commands.get_entity(entity) else {
			continue;
		};
		entity.despawn_recursive();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{components::dropdown::Dropdown, systems::dropdown::spawn_focused::DropdownUI};
	use bevy::{
		app::{App, Update},
		hierarchy::BuildWorldChildren,
		prelude::{Component, IntoSystem, Res, Resource},
	};
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Resource, Default)]
	struct _In(pub Focus);

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<_In>();
		app.add_systems(
			Update,
			(|new_active: Res<_In>| new_active.0.clone()).pipe(dropdown_despawn_unfocused),
		);

		app
	}

	#[test]
	fn despawn_dropdown_ui() {
		let mut app = setup();
		app.world.spawn_empty().with_children(|container| {
			container.spawn(Dropdown::default());
			container.spawn(DropdownUI);
		});

		app.world.insert_resource(_In(Focus::New(vec![])));

		app.update();

		let dropdown_uis = app
			.world
			.iter_entities()
			.find(|e| e.contains::<DropdownUI>());

		assert!(dropdown_uis.is_none());
	}

	#[test]
	fn do_not_despawn_not_dropdown_ui_entities() {
		#[derive(Component)]
		struct _Container;

		let mut app = setup();
		app.world.spawn(_Container).with_children(|container| {
			container.spawn(Dropdown::default());
			container.spawn(DropdownUI);
		});

		app.world.insert_resource(_In(Focus::New(vec![])));

		app.update();

		let non_dropdown_uis = app
			.world
			.iter_entities()
			.filter(|e| e.contains::<_Container>() || e.contains::<Dropdown>());

		assert_eq!(2, non_dropdown_uis.count());
	}

	#[test]
	fn despawn_dropdown_ui_recursively() {
		#[derive(Component)]
		struct _Child;

		let mut app = setup();
		app.world.spawn_empty().with_children(|container| {
			container.spawn(Dropdown::default());
			container.spawn(DropdownUI).with_children(|ui| {
				ui.spawn(_Child);
			});
		});

		app.world.insert_resource(_In(Focus::New(vec![])));

		app.update();

		let children = app.world.iter_entities().find(|e| e.contains::<_Child>());

		assert!(children.is_none());
	}

	#[test]
	fn do_not_despawn_dropdown_ui_of_active_dropdown() {
		let mut app = setup();
		let container = app.world.spawn_empty().id();
		let dropdown = app
			.world
			.spawn(Dropdown::default())
			.set_parent(container)
			.id();
		app.world.spawn(DropdownUI).set_parent(container);

		app.world.insert_resource(_In(Focus::New(vec![dropdown])));

		app.update();

		let dropdown_uis = app
			.world
			.iter_entities()
			.find(|e| e.contains::<DropdownUI>());

		assert!(dropdown_uis.is_some());
	}

	#[test]
	fn do_nothing_when_focus_unchanged() {
		let mut app = setup();
		app.world.spawn_empty().with_children(|container| {
			container.spawn(Dropdown::default());
			container.spawn(DropdownUI);
		});

		app.world.insert_resource(_In(Focus::Unchanged));

		app.update();

		let dropdown_uis = app
			.world
			.iter_entities()
			.find(|e| e.contains::<DropdownUI>());

		assert!(dropdown_uis.is_some());
	}
}
