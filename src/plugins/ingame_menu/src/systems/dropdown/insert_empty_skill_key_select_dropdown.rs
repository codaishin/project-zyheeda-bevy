use crate::components::{
	dropdown::Dropdown,
	key_select::{EmptySkill, KeySelect},
	KeySelectDropdownInsertCommand,
};
use bevy::prelude::{Commands, Entity, Query};
use common::traits::{
	iteration::IterFinite,
	try_insert_on::TryInsertOn,
	try_remove_from::TryRemoveFrom,
};

type InsertCommand<TKey> = KeySelectDropdownInsertCommand<EmptySkill, TKey>;

pub(crate) fn insert_empty_skill_key_select_dropdown<
	TKey: IterFinite + Copy + Sync + Send + 'static,
>(
	mut commands: Commands,
	insert_commands: Query<(Entity, &InsertCommand<TKey>)>,
) {
	for (entity, command) in &insert_commands {
		let items = TKey::iterator()
			.map(|key| KeySelect {
				extra: command.extra.clone(),
				key_button: entity,
				key_path: [command.key_path.clone(), vec![key]].concat(),
			})
			.collect();
		commands.try_insert_on(entity, Dropdown { items });
		commands.try_remove_from::<InsertCommand<TKey>>(entity);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{dropdown::Dropdown, key_select::KeySelect};
	use bevy::{
		app::{App, Update},
		prelude::Entity,
	};
	use common::{test_tools::utils::SingleThreadedApp, traits::iteration::Iter};

	#[derive(Debug, PartialEq, Clone, Copy)]
	enum _Key {
		A,
		B,
		C,
	}

	impl IterFinite for _Key {
		fn iterator() -> Iter<Self> {
			Iter(Some(_Key::A))
		}

		fn next(current: &Iter<Self>) -> Option<Self> {
			match current.0? {
				_Key::A => Some(_Key::B),
				_Key::B => Some(_Key::C),
				_Key::C => None,
			}
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, insert_empty_skill_key_select_dropdown::<_Key>);

		app
	}

	#[test]
	fn add_dropdown_for_each_equipment_key_mapped_to_dropdown_key() {
		let mut app = setup();
		let dropdown = app
			.world_mut()
			.spawn(InsertCommand {
				key_path: vec![_Key::A, _Key::B, _Key::C],
				extra: EmptySkill {
					button_entity: Entity::from_raw(42),
				},
			})
			.id();

		app.update();

		let dropdown = app.world().entity(dropdown);

		assert_eq!(
			Some(&Dropdown {
				items: vec![
					KeySelect {
						extra: EmptySkill {
							button_entity: Entity::from_raw(42)
						},
						key_button: dropdown.id(),
						key_path: vec![_Key::A, _Key::B, _Key::C, _Key::A]
					},
					KeySelect {
						extra: EmptySkill {
							button_entity: Entity::from_raw(42)
						},
						key_button: dropdown.id(),
						key_path: vec![_Key::A, _Key::B, _Key::C, _Key::B]
					},
					KeySelect {
						extra: EmptySkill {
							button_entity: Entity::from_raw(42)
						},
						key_button: dropdown.id(),
						key_path: vec![_Key::A, _Key::B, _Key::C, _Key::C]
					}
				]
			}),
			dropdown.get::<Dropdown<KeySelect<EmptySkill, _Key>>>(),
		)
	}

	#[test]
	fn remove_dropdown_insert_command() {
		let mut app = setup();
		let dropdown = app
			.world_mut()
			.spawn(InsertCommand {
				key_path: vec![_Key::A],
				extra: EmptySkill {
					button_entity: Entity::from_raw(42),
				},
			})
			.id();

		app.update();

		let dropdown = app.world().entity(dropdown);

		assert_eq!(None, dropdown.get::<InsertCommand<_Key>>(),)
	}
}
