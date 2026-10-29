use crate::components::dropdown::{Dropdown, DropdownEntityOf};
use bevy::prelude::*;
use common::prelude::*;

impl<T> Dropdown<T>
where
	T: ThreadSafe,
{
	pub(crate) fn expand(mut commands: ZyheedaCommands, dropdowns: Query<(Entity, &Self)>) {
		for (entity, dropdown) in dropdowns {
			for _ in &dropdown.items {
				commands.spawn((ChildOf(entity), DropdownEntityOf(entity)));
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::components::dropdown::{DropdownEntityOf, DropdownState};

	use super::*;
	use testing::{SingleThreadedApp, assert_related_count};

	struct _T;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, Dropdown::<_T>::expand);

		app
	}

	#[test]
	fn expand() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				DropdownState::Expanded,
				Dropdown {
					items: vec![_T, _T],
				},
			))
			.id();

		app.update();

		let children = assert_related_count!(ChildOf, 2, app, entity);
		assert_eq!(
			[
				Some(&DropdownEntityOf(entity)),
				Some(&DropdownEntityOf(entity))
			],
			children.map(|e| e.get::<DropdownEntityOf>()),
		);
	}

	#[test]
	fn do_nothing_if_collapsed() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				DropdownState::Expanded,
				Dropdown {
					items: vec![_T, _T],
				},
			))
			.id();

		app.update();

		let children = assert_related_count!(ChildOf, 2, app, entity);
		assert_eq!(
			[
				Some(&DropdownEntityOf(entity)),
				Some(&DropdownEntityOf(entity))
			],
			children.map(|e| e.get::<DropdownEntityOf>()),
		);
	}
}
