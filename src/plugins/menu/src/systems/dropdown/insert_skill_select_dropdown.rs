use crate::components::{
	dropdown::Dropdown,
	skill_button::{DropdownItem, SkillButton},
	SkillSelectDropdownInsertCommand,
};
use bevy::prelude::*;
use common::traits::{
	accessors::get::GetRef,
	try_insert_on::TryInsertOn,
	try_remove_from::TryRemoveFrom,
};
use player::components::player::Player;
use skills::{item::SkillItem, skills::Skill, slot_key::SlotKey};

pub(crate) fn insert_skill_select_dropdown<
	TEquipment: GetRef<SlotKey, SkillItem> + Component,
	TLayout: Sync + Send + 'static,
>(
	mut commands: Commands,
	dropdown_commands: Query<(Entity, &SkillSelectDropdownInsertCommand<SlotKey, TLayout>)>,
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
		commands.try_remove_from::<SkillSelectDropdownInsertCommand<SlotKey, TLayout>>(entity);
	}
}

fn compatible_skills<TEquipment: GetRef<SlotKey, SkillItem>, TLayout: Sync + Send + 'static>(
	command: &SkillSelectDropdownInsertCommand<SlotKey, TLayout>,
	slots: &TEquipment,
	skills: &Res<Assets<Skill>>,
) -> Option<Vec<SkillButton<DropdownItem<TLayout>>>> {
	let key = command.key_path.last()?;
	let item = slots.get(key)?;

	let mut seen = Vec::new();
	let skills = skills
		.iter()
		.filter(|(_, skill)| {
			if !skill.is_usable_with.contains(&item.content.item_type) {
				return false;
			}
			if seen.contains(skill) {
				return false;
			}

			seen.push(skill);
			true
		})
		.map(|(_, skill)| {
			SkillButton::<DropdownItem<TLayout>>::new(skill.clone(), command.key_path.clone())
		})
		.collect::<Vec<_>>();

	Some(skills)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::dropdown::Dropdown;
	use common::{components::Side, test_tools::utils::SingleThreadedApp};
	use skills::item::{item_type::SkillItemType, SkillItemContent};
	use std::collections::{HashMap, HashSet};
	use uuid::Uuid;

	#[derive(Debug, PartialEq)]
	struct _Layout;

	#[derive(Debug, PartialEq, Clone)]
	enum _DropdownKey {
		None,
		Ok,
	}

	#[derive(Component)]
	struct _Equipment(HashMap<SlotKey, SkillItem>);

	impl GetRef<SlotKey, SkillItem> for _Equipment {
		fn get(&self, key: &SlotKey) -> Option<&SkillItem> {
			self.0.get(key)
		}
	}

	fn setup<const N: usize>(skills: [Skill; N]) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut skill_assets = Assets::<Skill>::default();
		let _ = skills.map(|skill| skill_assets.add(skill));

		app.insert_resource(skill_assets);
		app.add_systems(Update, insert_skill_select_dropdown::<_Equipment, _Layout>);

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
				is_usable_with: HashSet::from([SkillItemType::Pistol]),
				icon: Some(image_a.clone()),
				..default()
			},
			Skill {
				name: "skill b".to_owned(),
				is_usable_with: HashSet::from([SkillItemType::Pistol, SkillItemType::Bracer]),
				icon: Some(image_b.clone()),
				..default()
			},
		];
		let mut app = setup(skills);

		app.world_mut().spawn((
			Player,
			_Equipment(HashMap::from([(
				SlotKey::BottomHand(Side::Right),
				SkillItem {
					content: SkillItemContent {
						item_type: SkillItemType::Pistol,
						..default()
					},
					..default()
				},
			)])),
		));
		let dropdown = app
			.world_mut()
			.spawn(SkillSelectDropdownInsertCommand::<SlotKey, _Layout>::new(
				vec![SlotKey::BottomHand(Side::Right)],
			))
			.id();

		app.update();

		let dropdown = app.world().entity(dropdown);

		assert_eq!(
			Some(&Dropdown {
				items: vec![
					SkillButton::<DropdownItem<_Layout>>::new(
						Skill {
							name: "skill a".to_owned(),
							is_usable_with: HashSet::from([SkillItemType::Pistol]),
							icon: Some(image_a.clone()),
							..default()
						},
						vec![SlotKey::BottomHand(Side::Right)],
					),
					SkillButton::<DropdownItem<_Layout>>::new(
						Skill {
							name: "skill b".to_owned(),
							is_usable_with: HashSet::from([
								SkillItemType::Pistol,
								SkillItemType::Bracer
							]),
							icon: Some(image_b.clone()),
							..default()
						},
						vec![SlotKey::BottomHand(Side::Right)],
					)
				]
			}),
			dropdown.get::<Dropdown<SkillButton<DropdownItem<_Layout>>>>()
		)
	}

	#[test]
	fn list_unique_skills() {
		let image_a = new_handle();
		let image_b = new_handle();
		let skills = [
			Skill {
				name: "skill a".to_owned(),
				is_usable_with: HashSet::from([SkillItemType::Pistol]),
				icon: Some(image_a.clone()),
				..default()
			},
			Skill {
				name: "skill b".to_owned(),
				is_usable_with: HashSet::from([SkillItemType::Pistol, SkillItemType::Bracer]),
				icon: Some(image_b.clone()),
				..default()
			},
			Skill {
				name: "skill b".to_owned(),
				is_usable_with: HashSet::from([SkillItemType::Pistol, SkillItemType::Bracer]),
				icon: Some(image_b.clone()),
				..default()
			},
		];
		let mut app = setup(skills);

		app.world_mut().spawn((
			Player,
			_Equipment(HashMap::from([(
				SlotKey::BottomHand(Side::Right),
				SkillItem {
					content: SkillItemContent {
						item_type: SkillItemType::Pistol,
						..default()
					},
					..default()
				},
			)])),
		));
		let dropdown = app
			.world_mut()
			.spawn(SkillSelectDropdownInsertCommand::<SlotKey, _Layout>::new(
				vec![SlotKey::BottomHand(Side::Right)],
			))
			.id();

		app.update();

		let dropdown = app.world().entity(dropdown);

		assert_eq!(
			Some(&Dropdown {
				items: vec![
					SkillButton::<DropdownItem<_Layout>>::new(
						Skill {
							name: "skill a".to_owned(),
							is_usable_with: HashSet::from([SkillItemType::Pistol]),
							icon: Some(image_a.clone()),
							..default()
						},
						vec![SlotKey::BottomHand(Side::Right)],
					),
					SkillButton::<DropdownItem<_Layout>>::new(
						Skill {
							name: "skill b".to_owned(),
							is_usable_with: HashSet::from([
								SkillItemType::Pistol,
								SkillItemType::Bracer
							]),
							icon: Some(image_b.clone()),
							..default()
						},
						vec![SlotKey::BottomHand(Side::Right)],
					)
				]
			}),
			dropdown.get::<Dropdown<SkillButton<DropdownItem<_Layout>>>>()
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
				is_usable_with: HashSet::from([SkillItemType::Pistol]),
				icon: Some(image_a.clone()),
				..default()
			},
			Skill {
				name: "skill b".to_owned(),
				is_usable_with: HashSet::from([SkillItemType::Pistol, SkillItemType::Bracer]),
				icon: Some(image_b.clone()),
				..default()
			},
		];
		let mut app = setup(skills.clone());

		app.world_mut().spawn((
			_NonPlayer,
			_Equipment(HashMap::from([(
				SlotKey::BottomHand(Side::Right),
				SkillItem {
					content: SkillItemContent {
						item_type: SkillItemType::Pistol,
						..default()
					},
					..default()
				},
			)])),
		));
		let dropdown = app
			.world_mut()
			.spawn(SkillSelectDropdownInsertCommand::<SlotKey, _Layout>::new(
				vec![SlotKey::BottomHand(Side::Right)],
			))
			.id();

		app.update();

		let dropdown = app.world().entity(dropdown);

		assert_eq!(
			None,
			dropdown.get::<Dropdown<SkillButton<DropdownItem<_Layout>>>>()
		);
	}

	#[test]
	fn remove_command() {
		let mut app = setup([]);

		app.world_mut().spawn((
			Player,
			_Equipment(HashMap::from([(
				SlotKey::BottomHand(Side::Right),
				SkillItem {
					content: SkillItemContent {
						item_type: SkillItemType::Pistol,
						..default()
					},
					..default()
				},
			)])),
		));
		let dropdown = app
			.world_mut()
			.spawn(SkillSelectDropdownInsertCommand::<SlotKey, _Layout>::new(
				vec![SlotKey::BottomHand(Side::Right)],
			))
			.id();

		app.update();

		let dropdown = app.world().entity(dropdown);

		assert_eq!(
			None,
			dropdown.get::<SkillSelectDropdownInsertCommand<SlotKey, _Layout>>()
		)
	}

	#[test]
	fn remove_command_when_not_item() {
		let mut app = setup([]);

		app.world_mut()
			.spawn((Player, _Equipment(HashMap::from([]))));
		let dropdown = app
			.world_mut()
			.spawn(SkillSelectDropdownInsertCommand::<SlotKey, _Layout>::new(
				vec![SlotKey::BottomHand(Side::Right)],
			))
			.id();

		app.update();

		let dropdown = app.world().entity(dropdown);

		assert_eq!(
			None,
			dropdown.get::<SkillSelectDropdownInsertCommand<SlotKey, _Layout>>()
		)
	}
}
