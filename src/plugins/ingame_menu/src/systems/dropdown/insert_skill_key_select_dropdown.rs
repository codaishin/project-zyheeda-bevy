use crate::components::{
	dropdown::Dropdown,
	key_select::{KeySelect, ReKeySkill},
	KeySelectDropdownInsertCommand,
	PreSelected,
};
use bevy::prelude::{Commands, Entity, Query, Res, Resource};
use common::traits::{
	iteration::IterFinite,
	map_value::MapForward,
	try_insert_on::TryInsertOn,
	try_remove_from::TryRemoveFrom,
};

type InsertCommand<TDropdownKey> =
	KeySelectDropdownInsertCommand<PreSelected<TDropdownKey>, TDropdownKey>;

pub(crate) fn insert_skill_key_select_dropdown<TDropdownKey, TEquipmentKey, TMap>(
	mut commands: Commands,
	insert_commands: Query<(Entity, &InsertCommand<TDropdownKey>)>,
	key_map: Res<TMap>,
) where
	TDropdownKey: PartialEq + Copy + Sync + Send + 'static,
	TEquipmentKey: Copy + IterFinite,
	TMap: MapForward<TEquipmentKey, TDropdownKey> + Resource,
{
	for (entity, command) in &insert_commands {
		let pre_selected = &command.extra;
		let items = TEquipmentKey::iterator()
			.map(|key| key_map.map_forward(key))
			.filter(|key| key != &pre_selected.key)
			.map(|key| KeySelect {
				extra: ReKeySkill { to: key },
				key_button: entity,
				key_path: command.key_path.clone(),
			})
			.collect();
		commands.try_insert_on(entity, Dropdown { items });
		commands.try_remove_from::<InsertCommand<TDropdownKey>>(entity);
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
	use common::{test_tools::utils::SingleThreadedApp, traits::iteration::Iter};

	#[derive(Debug, PartialEq, Clone, Copy)]
	enum _DropdownKey {
		A,
		B,
		C,
	}

	#[derive(Clone, Copy)]
	enum _EquipmentKey {
		A,
		B,
	}

	impl IterFinite for _EquipmentKey {
		fn iterator() -> Iter<Self> {
			Iter(Some(_EquipmentKey::A))
		}

		fn next(current: &Iter<Self>) -> Option<Self> {
			match current.0? {
				_EquipmentKey::A => Some(_EquipmentKey::B),
				_EquipmentKey::B => None,
			}
		}
	}

	#[derive(Resource, Default)]
	struct _Map;

	impl MapForward<_EquipmentKey, _DropdownKey> for _Map {
		fn map_forward(&self, value: _EquipmentKey) -> _DropdownKey {
			match value {
				_EquipmentKey::A => _DropdownKey::A,
				_EquipmentKey::B => _DropdownKey::B,
			}
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<_Map>();
		app.add_systems(
			Update,
			insert_skill_key_select_dropdown::<_DropdownKey, _EquipmentKey, _Map>,
		);

		app
	}

	#[test]
	fn add_dropdown_for_each_equipment_key_mapped_to_dropdown_key() {
		let mut app = setup();
		let dropdown = app
			.world_mut()
			.spawn(InsertCommand {
				key_path: vec![_DropdownKey::A, _DropdownKey::B, _DropdownKey::C],
				extra: PreSelected {
					key: _DropdownKey::C,
				},
			})
			.id();

		app.update();

		let dropdown = app.world().entity(dropdown);

		assert_eq!(
			Some(&Dropdown {
				items: vec![
					KeySelect {
						extra: ReKeySkill {
							to: _DropdownKey::A
						},
						key_button: dropdown.id(),
						key_path: vec![_DropdownKey::A, _DropdownKey::B, _DropdownKey::C,]
					},
					KeySelect {
						extra: ReKeySkill {
							to: _DropdownKey::B
						},
						key_button: dropdown.id(),
						key_path: vec![_DropdownKey::A, _DropdownKey::B, _DropdownKey::C,]
					}
				]
			}),
			dropdown.get::<Dropdown<KeySelect<ReKeySkill<_DropdownKey>, _DropdownKey>>>(),
		)
	}

	#[test]
	fn add_dropdown_without_already_selected() {
		let mut app = setup();
		let dropdown = app
			.world_mut()
			.spawn(InsertCommand {
				key_path: vec![_DropdownKey::A, _DropdownKey::B, _DropdownKey::C],
				extra: PreSelected {
					key: _DropdownKey::B,
				},
			})
			.id();

		app.update();

		let dropdown = app.world().entity(dropdown);

		assert_eq!(
			Some(&Dropdown {
				items: vec![KeySelect {
					extra: ReKeySkill {
						to: _DropdownKey::A
					},
					key_button: dropdown.id(),
					key_path: vec![_DropdownKey::A, _DropdownKey::B, _DropdownKey::C,]
				}]
			}),
			dropdown.get::<Dropdown<KeySelect<ReKeySkill<_DropdownKey>, _DropdownKey>>>(),
		)
	}

	#[test]
	fn remove_dropdown_insert_command() {
		let mut app = setup();
		let dropdown = app
			.world_mut()
			.spawn(InsertCommand {
				key_path: vec![_DropdownKey::A],
				extra: PreSelected {
					key: _DropdownKey::B,
				},
			})
			.id();

		app.update();

		let dropdown = app.world().entity(dropdown);

		assert_eq!(None, dropdown.get::<InsertCommand<_DropdownKey>>(),)
	}
}
