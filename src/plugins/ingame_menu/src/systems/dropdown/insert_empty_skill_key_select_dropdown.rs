use crate::{
	components::{
		dropdown::Dropdown,
		key_select::{EmptySkill, KeySelect},
		KeySelectDropdownInsertCommand,
	},
	systems::collect_all_keys::AllKeys,
};
use bevy::prelude::{Commands, Entity, KeyCode, Query, Res};
use common::traits::{try_insert_on::TryInsertOn, try_remove_from::TryRemoveFrom};

type InsertCommand = KeySelectDropdownInsertCommand<EmptySkill, KeyCode>;

pub(crate) fn insert_empty_skill_key_select_dropdown(
	mut commands: Commands,
	insert_commands: Query<(Entity, &InsertCommand)>,
	all_keys: Res<AllKeys<KeyCode>>,
) {
	for (entity, command) in &insert_commands {
		let items = all_keys
			.keys()
			.iter()
			.map(|key| KeySelect {
				extra: command.extra.clone(),
				key_button: entity,
				key_path: [command.key_path.clone(), vec![*key]].concat(),
			})
			.collect();
		commands.try_insert_on(entity, Dropdown { items });
		commands.try_remove_from::<InsertCommand>(entity);
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
	use common::test_tools::utils::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(AllKeys::new(vec![
			KeyCode::KeyA,
			KeyCode::KeyB,
			KeyCode::KeyC,
		]));
		app.add_systems(Update, insert_empty_skill_key_select_dropdown);

		app
	}

	#[test]
	fn add_dropdown_for_each_equipment_key_mapped_to_dropdown_key() {
		let mut app = setup();
		let dropdown = app
			.world_mut()
			.spawn(InsertCommand {
				key_path: vec![KeyCode::KeyA, KeyCode::KeyB, KeyCode::KeyC],
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
						key_path: vec![KeyCode::KeyA, KeyCode::KeyB, KeyCode::KeyC, KeyCode::KeyA]
					},
					KeySelect {
						extra: EmptySkill {
							button_entity: Entity::from_raw(42)
						},
						key_button: dropdown.id(),
						key_path: vec![KeyCode::KeyA, KeyCode::KeyB, KeyCode::KeyC, KeyCode::KeyB]
					},
					KeySelect {
						extra: EmptySkill {
							button_entity: Entity::from_raw(42)
						},
						key_button: dropdown.id(),
						key_path: vec![KeyCode::KeyA, KeyCode::KeyB, KeyCode::KeyC, KeyCode::KeyC]
					}
				]
			}),
			dropdown.get::<Dropdown<KeySelect<EmptySkill, KeyCode>>>(),
		)
	}

	#[test]
	fn remove_dropdown_insert_command() {
		let mut app = setup();
		let dropdown = app
			.world_mut()
			.spawn(InsertCommand {
				key_path: vec![KeyCode::KeyA],
				extra: EmptySkill {
					button_entity: Entity::from_raw(42),
				},
			})
			.id();

		app.update();

		let dropdown = app.world().entity(dropdown);

		assert_eq!(None, dropdown.get::<InsertCommand>(),)
	}
}
