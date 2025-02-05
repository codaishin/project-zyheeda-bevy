use crate::components::{
	dropdown::Dropdown,
	skill_button::{DropdownItem, SkillButton},
	SkillSelectDropdownInsertCommand,
};
use bevy::prelude::*;
use common::{
	tools::{item_type::ItemType, slot_key::SlotKey},
	traits::{
		accessors::get::{GetField, GetFieldRef, Getter, GetterRef},
		handles_equipment::{CompatibleItems, SingleAccess},
		thread_safe::ThreadSafe,
		try_insert_on::TryInsertOn,
		try_remove_from::TryRemoveFrom,
	},
};

impl<T> DropdownSkillSelectInsert for T {}

pub(crate) trait DropdownSkillSelectInsert {
	fn dropdown_skill_select_insert<TPlayer, TSkill, TSlots>(
		mut commands: Commands,
		dropdown_commands: Query<(Entity, &SkillSelectDropdownInsertCommand<SlotKey, Self>)>,
		slots: Query<&TSlots, With<TPlayer>>,
		items: Res<Assets<TSlots::TItem>>,
		skills: Res<Assets<TSkill>>,
	) where
		Self: ThreadSafe + Sized,
		TSlots: SingleAccess<TKey = SlotKey> + Component,
		TSlots::TItem: Getter<ItemType>,
		TPlayer: Component,
		TSkill: Asset + PartialEq + Clone + GetterRef<CompatibleItems>,
	{
		let Ok(slots) = slots.get_single() else {
			return;
		};

		for (entity, command) in &dropdown_commands {
			if let Some(items) = compatible_skills(command, slots, &items, &skills) {
				commands.try_insert_on(entity, Dropdown { items });
			}
			commands.try_remove_from::<SkillSelectDropdownInsertCommand<SlotKey, Self>>(entity);
		}
	}
}

fn compatible_skills<TSlots, TLayout, TSkill>(
	command: &SkillSelectDropdownInsertCommand<SlotKey, TLayout>,
	slots: &TSlots,
	items: &Assets<TSlots::TItem>,
	skills: &Assets<TSkill>,
) -> Option<Vec<SkillButton<DropdownItem<TLayout>, TSkill>>>
where
	TSlots: SingleAccess<TKey = SlotKey>,
	TSlots::TItem: Getter<ItemType>,
	TLayout: ThreadSafe,
	TSkill: Asset + PartialEq + Clone + GetterRef<CompatibleItems>,
{
	let key = command.key_path.last()?;
	let item = slots
		.single_access(key)
		.ok()
		.and_then(|handle| handle.as_ref())
		.and_then(|handle| items.get(handle))?;
	let item_type = ItemType::get_field(item);

	let mut seen = Vec::new();
	let skills = skills
		.iter()
		.filter(|(_, skill)| {
			let CompatibleItems(is_usable_with) = CompatibleItems::get_field_ref(*skill);
			if !is_usable_with.contains(&item_type) {
				return false;
			}
			if seen.contains(skill) {
				return false;
			}

			seen.push(skill);
			true
		})
		.map(|(_, skill)| {
			SkillButton::<DropdownItem<TLayout>, TSkill>::new(
				skill.clone(),
				command.key_path.clone(),
			)
		})
		.collect::<Vec<_>>();

	Some(skills)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::dropdown::Dropdown;
	use common::{
		test_tools::utils::SingleThreadedApp,
		tools::{item_type::ItemType, slot_key::Side},
		traits::handles_equipment::KeyOutOfBounds,
	};
	use std::collections::{HashMap, HashSet};

	#[derive(Asset, TypePath, Debug, PartialEq, Clone)]
	struct _Skill(CompatibleItems);

	impl GetterRef<CompatibleItems> for _Skill {
		fn get(&self) -> &CompatibleItems {
			&self.0
		}
	}

	#[derive(Component)]
	struct _Player;

	#[derive(Debug, PartialEq)]
	struct _Layout;

	#[derive(Debug, PartialEq, Clone)]
	enum _DropdownKey {
		None,
		Ok,
	}

	#[derive(Asset, TypePath, Debug, PartialEq)]
	struct _Item(ItemType);

	impl Getter<ItemType> for _Item {
		fn get(&self) -> ItemType {
			self.0
		}
	}

	#[derive(Component)]
	struct _Slots(HashMap<SlotKey, Option<Handle<_Item>>>);

	impl SingleAccess for _Slots {
		type TKey = SlotKey;
		type TItem = _Item;

		fn single_access(
			&self,
			key: &Self::TKey,
		) -> Result<&Option<Handle<Self::TItem>>, KeyOutOfBounds> {
			let Some(item) = self.0.get(key) else {
				return Err(KeyOutOfBounds);
			};

			Ok(item)
		}
	}

	fn setup_skills_and_equipment<const S: usize, const E: usize>(
		skills: [_Skill; S],
		equipment: [(SlotKey, _Item); E],
	) -> (_Slots, Assets<_Item>, Assets<_Skill>) {
		let mut item_assets = Assets::default();
		let mut skill_assets = Assets::default();

		for skill in skills {
			skill_assets.add(skill);
		}

		let equipment = equipment
			.into_iter()
			.map(|(key, item)| (key, Some(item_assets.add(item))))
			.collect();

		(_Slots(equipment), item_assets, skill_assets)
	}

	fn setup_app<const S: usize, const E: usize>(
		agent: impl Component,
		skills: [_Skill; S],
		slots: [(SlotKey, _Item); E],
	) -> App {
		let (equipment, items, skills) = setup_skills_and_equipment(skills, slots);
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(items);
		app.insert_resource(skills);
		app.add_systems(
			Update,
			_Layout::dropdown_skill_select_insert::<_Player, _Skill, _Slots>,
		);
		app.world_mut().spawn((agent, equipment));

		app
	}

	#[test]
	fn list_compatible_skills() {
		let mut app = setup_app(
			_Player,
			[
				_Skill(CompatibleItems(HashSet::from([ItemType::Pistol]))),
				_Skill(CompatibleItems(HashSet::from([
					ItemType::Pistol,
					ItemType::Bracer,
				]))),
			],
			[(SlotKey::BottomHand(Side::Right), _Item(ItemType::Pistol))],
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
					SkillButton::<DropdownItem<_Layout>, _Skill>::new(
						_Skill(CompatibleItems(HashSet::from([ItemType::Pistol]),)),
						vec![SlotKey::BottomHand(Side::Right)],
					),
					SkillButton::<DropdownItem<_Layout>, _Skill>::new(
						_Skill(CompatibleItems(HashSet::from([
							ItemType::Pistol,
							ItemType::Bracer
						]),)),
						vec![SlotKey::BottomHand(Side::Right)],
					)
				]
			}),
			dropdown.get::<Dropdown<SkillButton<DropdownItem<_Layout>, _Skill>>>()
		)
	}

	#[test]
	fn list_unique_skills() {
		let mut app = setup_app(
			_Player,
			[
				_Skill(CompatibleItems(HashSet::from([ItemType::Pistol]))),
				_Skill(CompatibleItems(HashSet::from([
					ItemType::Pistol,
					ItemType::Bracer,
				]))),
				_Skill(CompatibleItems(HashSet::from([
					ItemType::Pistol,
					ItemType::Bracer,
				]))),
			],
			[(SlotKey::BottomHand(Side::Right), _Item(ItemType::Pistol))],
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
					SkillButton::<DropdownItem<_Layout>, _Skill>::new(
						_Skill(CompatibleItems(HashSet::from([ItemType::Pistol]),)),
						vec![SlotKey::BottomHand(Side::Right)],
					),
					SkillButton::<DropdownItem<_Layout>, _Skill>::new(
						_Skill(CompatibleItems(HashSet::from([
							ItemType::Pistol,
							ItemType::Bracer
						]),)),
						vec![SlotKey::BottomHand(Side::Right)],
					)
				]
			}),
			dropdown.get::<Dropdown<SkillButton<DropdownItem<_Layout>, _Skill>>>()
		)
	}

	#[test]
	fn do_not_list_compatible_skills_of_non_player() {
		#[derive(Component)]
		struct _NonPlayer;

		let mut app = setup_app(
			_NonPlayer,
			[
				_Skill(CompatibleItems(HashSet::from([ItemType::Pistol]))),
				_Skill(CompatibleItems(HashSet::from([
					ItemType::Pistol,
					ItemType::Bracer,
				]))),
			],
			[(SlotKey::BottomHand(Side::Right), _Item(ItemType::Pistol))],
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
			dropdown.get::<Dropdown<SkillButton<DropdownItem<_Layout>, _Skill>>>()
		);
	}

	#[test]
	fn remove_command() {
		let mut app = setup_app(
			_Player,
			[],
			[(SlotKey::BottomHand(Side::Right), _Item(ItemType::Pistol))],
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
		let mut app = setup_app(_Player, [], []);
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
