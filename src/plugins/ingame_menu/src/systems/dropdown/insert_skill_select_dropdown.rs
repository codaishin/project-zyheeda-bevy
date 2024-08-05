use crate::components::{
	dropdown::Dropdown,
	skill_descriptor::SkillDescriptor,
	SkillSelectDropdownInsertCommand,
};
use bevy::{
	asset::{Assets, Handle},
	prelude::{Commands, Component, Entity, Query, Res, With},
};
use common::{
	components::Player,
	traits::{get::Get, try_insert_on::TryInsertOn, try_remove_from::TryRemoveFrom},
};
use skills::{
	items::{slot_key::SlotKey, Item},
	skills::Skill,
};

pub(crate) fn insert_skill_select_dropdown<
	TEquipment: Get<SlotKey, Item<Handle<Skill>>> + Component,
>(
	mut commands: Commands,
	dropdown_commands: Query<(Entity, &SkillSelectDropdownInsertCommand)>,
	slots: Query<&TEquipment, With<Player>>,
	skills: Res<Assets<Skill>>,
) {
	let Ok(slots) = slots.get_single() else {
		return;
	};

	for (entity, command) in &dropdown_commands {
		if let Some(items) = compatible_skills(command, slots, &skills) {
			commands.try_insert_on(entity, Dropdown { items });
		}
		commands.try_remove_from::<SkillSelectDropdownInsertCommand>(entity);
	}
}

fn compatible_skills<TEquipment: Get<SlotKey, Item<Handle<Skill>>>>(
	command: &SkillSelectDropdownInsertCommand,
	slots: &TEquipment,
	skills: &Res<Assets<Skill>>,
) -> Option<Vec<SkillDescriptor>> {
	let key = command.key_path.last()?;
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
		.map(|(_, skill)| {
			SkillDescriptor::new_dropdown_item(skill.clone(), command.key_path.clone())
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
	use common::{
		components::{Player, Side},
		test_tools::utils::SingleThreadedApp,
	};
	use skills::items::ItemType;
	use std::collections::{HashMap, HashSet};
	use uuid::Uuid;

	#[derive(Debug, PartialEq, Clone)]
	enum _DropdownKey {
		None,
		Ok,
	}

	#[derive(Component)]
	struct _Equipment(HashMap<SlotKey, Item<Handle<Skill>>>);

	impl Get<SlotKey, Item<Handle<Skill>>> for _Equipment {
		fn get(&self, key: &SlotKey) -> Option<&Item<Handle<Skill>>> {
			self.0.get(key)
		}
	}

	fn setup<const N: usize>(skills: [Skill; N]) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut skill_assets = Assets::<Skill>::default();
		let _ = skills.map(|skill| skill_assets.add(skill));

		app.insert_resource(skill_assets);
		app.add_systems(Update, insert_skill_select_dropdown::<_Equipment>);

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
				SlotKey::Hand(Side::Main),
				Item {
					item_type: HashSet::from([ItemType::Pistol]),
					..default()
				},
			)])),
		));
		let dropdown = app
			.world_mut()
			.spawn(SkillSelectDropdownInsertCommand {
				key_path: vec![SlotKey::Hand(Side::Main)],
			})
			.id();

		app.update();

		let dropdown = app.world().entity(dropdown);

		assert_eq!(
			Some(&Dropdown {
				items: vec![
					SkillDescriptor::new_dropdown_item(
						Skill {
							name: "skill a".to_owned(),
							is_usable_with: HashSet::from([ItemType::Pistol]),
							icon: Some(image_a.clone()),
							..default()
						},
						vec![SlotKey::Hand(Side::Main)],
					),
					SkillDescriptor::new_dropdown_item(
						Skill {
							name: "skill b".to_owned(),
							is_usable_with: HashSet::from([ItemType::Pistol, ItemType::Bracer]),
							icon: Some(image_b.clone()),
							..default()
						},
						vec![SlotKey::Hand(Side::Main)],
					)
				]
			}),
			dropdown.get::<Dropdown<SkillDescriptor>>()
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
				SlotKey::Hand(Side::Main),
				Item {
					item_type: HashSet::from([ItemType::Pistol]),
					..default()
				},
			)])),
		));
		let dropdown = app
			.world_mut()
			.spawn(SkillSelectDropdownInsertCommand {
				key_path: vec![SlotKey::Hand(Side::Main)],
			})
			.id();

		app.update();

		let dropdown = app.world().entity(dropdown);

		assert_eq!(None, dropdown.get::<Dropdown<SkillDescriptor>>());
	}

	#[test]
	fn remove_command() {
		let mut app = setup([]);

		app.world_mut().spawn((
			Player,
			_Equipment(HashMap::from([(
				SlotKey::Hand(Side::Main),
				Item {
					item_type: HashSet::from([ItemType::Pistol]),
					..default()
				},
			)])),
		));
		let dropdown = app
			.world_mut()
			.spawn(SkillSelectDropdownInsertCommand {
				key_path: vec![SlotKey::Hand(Side::Main)],
			})
			.id();

		app.update();

		let dropdown = app.world().entity(dropdown);

		assert_eq!(None, dropdown.get::<SkillSelectDropdownInsertCommand>())
	}

	#[test]
	fn remove_command_when_not_item() {
		let mut app = setup([]);

		app.world_mut()
			.spawn((Player, _Equipment(HashMap::from([]))));
		let dropdown = app
			.world_mut()
			.spawn(SkillSelectDropdownInsertCommand {
				key_path: vec![SlotKey::Hand(Side::Main)],
			})
			.id();

		app.update();

		let dropdown = app.world().entity(dropdown);

		assert_eq!(None, dropdown.get::<SkillSelectDropdownInsertCommand>())
	}
}
