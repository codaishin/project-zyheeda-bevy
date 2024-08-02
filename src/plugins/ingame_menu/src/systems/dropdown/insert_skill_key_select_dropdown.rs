use crate::{
	components::{
		dropdown::Dropdown,
		key_select::{KeySelect, ReKeySkill},
		KeySelectDropdownInsertCommand,
		PreSelected,
	},
	systems::collect_all_keys::AllKeys,
};
use bevy::prelude::{Commands, Entity, KeyCode, Query, Res};
use common::traits::{try_insert_on::TryInsertOn, try_remove_from::TryRemoveFrom};

type InsertCommand = KeySelectDropdownInsertCommand<PreSelected<KeyCode>, KeyCode>;

pub(crate) fn insert_skill_key_select_dropdown(
	mut commands: Commands,
	insert_commands: Query<(Entity, &InsertCommand)>,
	all_keys: Res<AllKeys<KeyCode>>,
) {
	for (entity, command) in &insert_commands {
		let pre_selected = &command.extra;
		let items = all_keys
			.keys()
			.iter()
			.filter(|key| key != &&pre_selected.key)
			.map(|key| KeySelect {
				extra: ReKeySkill { to: *key },
				key_button: entity,
				key_path: command.key_path.clone(),
			})
			.collect();
		commands.try_insert_on(entity, Dropdown { items });
		commands.try_remove_from::<InsertCommand>(entity);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{
		dropdown::Dropdown,
		key_select::{KeySelect, ReKeySkill},
	};
	use bevy::app::{App, Update};
	use common::test_tools::utils::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(AllKeys::new(vec![
			KeyCode::KeyA,
			KeyCode::KeyB,
			KeyCode::KeyC,
		]));
		app.add_systems(Update, insert_skill_key_select_dropdown);

		app
	}

	#[test]
	fn add_dropdown_key_select_without_pre_selected() {
		let mut app = setup();
		let dropdown = app
			.world_mut()
			.spawn(InsertCommand {
				key_path: vec![KeyCode::KeyA, KeyCode::KeyB, KeyCode::KeyC],
				extra: PreSelected { key: KeyCode::KeyB },
			})
			.id();

		app.update();

		let dropdown = app.world().entity(dropdown);

		assert_eq!(
			Some(&Dropdown {
				items: vec![
					KeySelect {
						extra: ReKeySkill { to: KeyCode::KeyA },
						key_button: dropdown.id(),
						key_path: vec![KeyCode::KeyA, KeyCode::KeyB, KeyCode::KeyC]
					},
					KeySelect {
						extra: ReKeySkill { to: KeyCode::KeyC },
						key_button: dropdown.id(),
						key_path: vec![KeyCode::KeyA, KeyCode::KeyB, KeyCode::KeyC]
					}
				]
			}),
			dropdown.get::<Dropdown<KeySelect<ReKeySkill<KeyCode>, KeyCode>>>(),
		)
	}

	#[test]
	fn remove_dropdown_insert_command() {
		let mut app = setup();
		let dropdown = app
			.world_mut()
			.spawn(InsertCommand {
				key_path: vec![KeyCode::KeyA],
				extra: PreSelected { key: KeyCode::KeyB },
			})
			.id();

		app.update();

		let dropdown = app.world().entity(dropdown);

		assert_eq!(None, dropdown.get::<InsertCommand>(),)
	}
}
