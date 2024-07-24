use crate::components::{
	dropdown::{skill_select::SkillSelect, Dropdown},
	SkillSelectDropdownCommand,
};
use bevy::{
	asset::{Assets, Handle},
	prelude::{Commands, Component, Entity, Query, Res, Resource, With},
};
use common::{
	components::Player,
	traits::{
		get::Get,
		map_value::TryMapBackwards,
		try_insert_on::TryInsertOn,
		try_remove_from::TryRemoveFrom,
	},
};
use skills::{items::Item, skills::Skill};

pub(crate) fn skill_select_dropdown<
	TDropdownKey: Clone + Sync + Send + 'static,
	TEquipmentKey: Clone + Sync + Send + 'static,
	TMap: TryMapBackwards<TDropdownKey, TEquipmentKey> + Resource,
	TEquipment: Get<TEquipmentKey, Item<Handle<Skill>>> + Component,
>(
	mut commands: Commands,
	dropdown_commands: Query<(Entity, &SkillSelectDropdownCommand<TDropdownKey>)>,
	slots: Query<&TEquipment, With<Player>>,
	skills: Res<Assets<Skill>>,
	key_map: Res<TMap>,
) {
	let Ok(slots) = slots.get_single() else {
		return;
	};

	for (entity, command) in &dropdown_commands {
		if let Some(items) = compatible_skills(command, slots, &skills, key_map.as_ref()) {
			commands.try_insert_on(entity, Dropdown { items });
		}
		commands.try_remove_from::<SkillSelectDropdownCommand<TDropdownKey>>(entity);
	}
}

fn compatible_skills<
	TDropdownKey: Clone + Sync + Send + 'static,
	TEquipmentKey: Clone + Sync + Send + 'static,
	TMap: TryMapBackwards<TDropdownKey, TEquipmentKey> + Resource,
	TEquipment: Get<TEquipmentKey, Item<Handle<Skill>>>,
>(
	command: &SkillSelectDropdownCommand<TDropdownKey>,
	slots: &TEquipment,
	skills: &Res<Assets<Skill>>,
	key_map: &TMap,
) -> Option<Vec<SkillSelect<TEquipmentKey>>> {
	let key_path = command
		.key_path
		.iter()
		.cloned()
		.filter_map(|key| key_map.try_map_backwards(key))
		.collect::<Vec<_>>();
	let key = key_path.last()?;
	let item = slots.get(key)?;
	let skills = skills
		.iter()
		.filter(|(_, skill)| {
			skill
				.is_usable_with
				.intersection(&item.item_type)
				.next()
				.is_some()
		})
		.map(|(_, skill)| SkillSelect {
			skill: skill.clone(),
			key_path: key_path.clone(),
		})
		.collect::<Vec<_>>();

	Some(skills)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::dropdown::Dropdown;
	use bevy::{
		app::{App, Update},
		asset::{Asset, AssetId, Assets},
		prelude::{default, Component},
	};
	use common::{components::Player, test_tools::utils::SingleThreadedApp};
	use skills::items::ItemType;
	use std::collections::{HashMap, HashSet};
	use uuid::Uuid;

	#[derive(Debug, PartialEq, Clone)]
	enum _DropdownKey {
		None,
		Ok,
	}

	#[derive(Debug, Clone, PartialEq, Eq, Hash)]
	struct _EquipmentKey;

	#[derive(Component)]
	struct _Equipment(HashMap<_EquipmentKey, Item<Handle<Skill>>>);

	impl Get<_EquipmentKey, Item<Handle<Skill>>> for _Equipment {
		fn get(&self, key: &_EquipmentKey) -> Option<&Item<Handle<Skill>>> {
			self.0.get(key)
		}
	}

	#[derive(Default, Resource)]
	struct _Map;

	impl TryMapBackwards<_DropdownKey, _EquipmentKey> for _Map {
		fn try_map_backwards(&self, value: _DropdownKey) -> Option<_EquipmentKey> {
			match value {
				_DropdownKey::Ok => Some(_EquipmentKey),
				_DropdownKey::None => None,
			}
		}
	}

	fn setup<const N: usize>(skills: [Skill; N]) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut skill_assets = Assets::<Skill>::default();
		let _ = skills.map(|skill| skill_assets.add(skill));

		app.init_resource::<_Map>();
		app.insert_resource(skill_assets);
		app.add_systems(
			Update,
			skill_select_dropdown::<_DropdownKey, _EquipmentKey, _Map, _Equipment>,
		);

		app
	}

	fn new_handle<T: Asset>() -> Handle<T> {
		Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		})
	}

	#[test]
	fn list_compatible_skills() {
		let image_a = new_handle();
		let image_b = new_handle();
		let skills = [
			Skill {
				name: "skill a".to_owned(),
				is_usable_with: HashSet::from([ItemType::Pistol]),
				icon: Some(image_a.clone()),
				..default()
			},
			Skill {
				name: "skill b".to_owned(),
				is_usable_with: HashSet::from([ItemType::Pistol, ItemType::Bracer]),
				icon: Some(image_b.clone()),
				..default()
			},
		];
		let mut app = setup(skills);

		app.world_mut().spawn((
			Player,
			_Equipment(HashMap::from([(
				_EquipmentKey,
				Item {
					item_type: HashSet::from([ItemType::Pistol]),
					..default()
				},
			)])),
		));
		let dropdown = app
			.world_mut()
			.spawn(SkillSelectDropdownCommand {
				key_path: vec![_DropdownKey::Ok],
			})
			.id();

		app.update();

		let dropdown = app.world().entity(dropdown);

		assert_eq!(
			Some(&Dropdown {
				items: vec![
					SkillSelect {
						skill: Skill {
							name: "skill a".to_owned(),
							is_usable_with: HashSet::from([ItemType::Pistol]),
							icon: Some(image_a.clone()),
							..default()
						},
						key_path: vec![_EquipmentKey],
					},
					SkillSelect {
						skill: Skill {
							name: "skill b".to_owned(),
							is_usable_with: HashSet::from([ItemType::Pistol, ItemType::Bracer]),
							icon: Some(image_b.clone()),
							..default()
						},
						key_path: vec![_EquipmentKey],
					}
				]
			}),
			dropdown.get::<Dropdown<SkillSelect<_EquipmentKey>>>()
		)
	}

	#[test]
	fn do_not_list_compatible_skills_of_non_player() {
		#[derive(Component)]
		struct _NonPlayer;

		let image_a = new_handle();
		let image_b = new_handle();
		let skills = [
			Skill {
				name: "skill a".to_owned(),
				is_usable_with: HashSet::from([ItemType::Pistol]),
				icon: Some(image_a.clone()),
				..default()
			},
			Skill {
				name: "skill b".to_owned(),
				is_usable_with: HashSet::from([ItemType::Pistol, ItemType::Bracer]),
				icon: Some(image_b.clone()),
				..default()
			},
		];
		let mut app = setup(skills.clone());

		app.world_mut().spawn((
			_NonPlayer,
			_Equipment(HashMap::from([(
				_EquipmentKey,
				Item {
					item_type: HashSet::from([ItemType::Pistol]),
					..default()
				},
			)])),
		));
		let dropdown = app
			.world_mut()
			.spawn(SkillSelectDropdownCommand {
				key_path: vec![_DropdownKey::Ok],
			})
			.id();

		app.update();

		let dropdown = app.world().entity(dropdown);

		assert_eq!(None, dropdown.get::<Dropdown<SkillSelect<_EquipmentKey>>>());
	}

	#[test]
	fn remove_command() {
		let mut app = setup([]);

		app.world_mut().spawn((
			Player,
			_Equipment(HashMap::from([(
				_EquipmentKey,
				Item {
					item_type: HashSet::from([ItemType::Pistol]),
					..default()
				},
			)])),
		));
		let dropdown = app
			.world_mut()
			.spawn(SkillSelectDropdownCommand {
				key_path: vec![_DropdownKey::Ok],
			})
			.id();

		app.update();

		let dropdown = app.world().entity(dropdown);

		assert_eq!(
			None,
			dropdown.get::<SkillSelectDropdownCommand<_DropdownKey>>()
		)
	}

	#[test]
	fn remove_command_when_key_cannot_be_mapped() {
		let mut app = setup([]);

		app.world_mut().spawn((
			Player,
			_Equipment(HashMap::from([(
				_EquipmentKey,
				Item {
					item_type: HashSet::from([ItemType::Pistol]),
					..default()
				},
			)])),
		));
		let dropdown = app
			.world_mut()
			.spawn(SkillSelectDropdownCommand {
				key_path: vec![_DropdownKey::None],
			})
			.id();

		app.update();

		let dropdown = app.world().entity(dropdown);

		assert_eq!(
			None,
			dropdown.get::<SkillSelectDropdownCommand<_DropdownKey>>()
		)
	}

	#[test]
	fn remove_command_when_not_item() {
		let mut app = setup([]);

		app.world_mut()
			.spawn((Player, _Equipment(HashMap::from([]))));
		let dropdown = app
			.world_mut()
			.spawn(SkillSelectDropdownCommand {
				key_path: vec![_DropdownKey::Ok],
			})
			.id();

		app.update();

		let dropdown = app.world().entity(dropdown);

		assert_eq!(
			None,
			dropdown.get::<SkillSelectDropdownCommand<_DropdownKey>>()
		)
	}
}
