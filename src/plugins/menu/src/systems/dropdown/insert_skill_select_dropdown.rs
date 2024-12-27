use crate::components::{
	dropdown::Dropdown,
	skill_button::{DropdownItem, SkillButton},
	SkillSelectDropdownInsertCommand,
};
use bevy::prelude::*;
use common::{
	tools::slot_key::SlotKey,
	traits::{accessors::get::GetRef, try_insert_on::TryInsertOn, try_remove_from::TryRemoveFrom},
};
use player::components::player::Player;
use skills::{item::Item, skills::Skill};

pub(crate) fn insert_skill_select_dropdown<
	TEquipment: GetRef<SlotKey, Handle<Item>> + Component,
	TLayout: Sync + Send + 'static,
>(
	mut commands: Commands,
	dropdown_commands: Query<(Entity, &SkillSelectDropdownInsertCommand<SlotKey, TLayout>)>,
	slots: Query<&TEquipment, With<Player>>,
	items: Res<Assets<Item>>,
	skills: Res<Assets<Skill>>,
) {
	let Ok(slots) = slots.get_single() else {
		return;
	};

	for (entity, command) in &dropdown_commands {
		if let Some(items) = compatible_skills(command, slots, &items, &skills) {
			commands.try_insert_on(entity, Dropdown { items });
		}
		commands.try_remove_from::<SkillSelectDropdownInsertCommand<SlotKey, TLayout>>(entity);
	}
}

fn compatible_skills<TEquipment: GetRef<SlotKey, Handle<Item>>, TLayout: Sync + Send + 'static>(
	command: &SkillSelectDropdownInsertCommand<SlotKey, TLayout>,
	slots: &TEquipment,
	items: &Assets<Item>,
	skills: &Assets<Skill>,
) -> Option<Vec<SkillButton<DropdownItem<TLayout>>>> {
	let key = command.key_path.last()?;
	let item = slots.get(key).and_then(|item| items.get(item))?;

	let mut seen = Vec::new();
	let skills = skills
		.iter()
		.filter(|(_, skill)| {
			if !skill.is_usable_with.contains(&item.item_type) {
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
	use common::{test_tools::utils::SingleThreadedApp, tools::slot_key::Side};
	use skills::item::item_type::SkillItemType;
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
	struct _Equipment(HashMap<SlotKey, Handle<Item>>);

	impl GetRef<SlotKey, Handle<Item>> for _Equipment {
		fn get(&self, key: &SlotKey) -> Option<&Handle<Item>> {
			self.0.get(key)
		}
	}

	fn setup_skills_and_equipment<const S: usize, const E: usize>(
		skills: [Skill; S],
		equipment: [(SlotKey, Item); E],
	) -> (_Equipment, Assets<Item>, Assets<Skill>) {
		let mut item_assets = Assets::default();
		let mut skill_assets = Assets::default();

		for skill in skills {
			skill_assets.add(skill);
		}

		let equipment = equipment
			.into_iter()
			.map(|(key, item)| (key, item_assets.add(item)))
			.collect();

		(_Equipment(equipment), item_assets, skill_assets)
	}

	fn setup_app<const S: usize, const E: usize>(
		agent: impl Component,
		skills: [Skill; S],
		equipment: [(SlotKey, Item); E],
	) -> App {
		let (equipment, items, skills) = setup_skills_and_equipment(skills, equipment);
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(items);
		app.insert_resource(skills);
		app.add_systems(Update, insert_skill_select_dropdown::<_Equipment, _Layout>);
		app.world_mut().spawn((agent, equipment));

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
		let mut app = setup_app(
			Player,
			[
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
			],
			[(
				SlotKey::BottomHand(Side::Right),
				Item {
					item_type: SkillItemType::Pistol,
					..default()
				},
			)],
		);
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
		let mut app = setup_app(
			Player,
			[
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
			],
			[(
				SlotKey::BottomHand(Side::Right),
				Item {
					item_type: SkillItemType::Pistol,
					..default()
				},
			)],
		);
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
		let mut app = setup_app(
			_NonPlayer,
			[
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
			],
			[(
				SlotKey::BottomHand(Side::Right),
				Item {
					item_type: SkillItemType::Pistol,
					..default()
				},
			)],
		);
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
		let mut app = setup_app(
			Player,
			[],
			[(
				SlotKey::BottomHand(Side::Right),
				Item {
					item_type: SkillItemType::Pistol,
					..default()
				},
			)],
		);
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
		let mut app = setup_app(Player, [], []);
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
