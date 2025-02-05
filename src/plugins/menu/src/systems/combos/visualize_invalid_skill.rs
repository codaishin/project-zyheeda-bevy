use crate::{
	components::skill_button::{DropdownTrigger, SkillButton},
	traits::InsertContentOn,
};
use bevy::{ecs::system::EntityCommands, prelude::*};
use common::{
	tools::{item_type::ItemType, slot_key::SlotKey},
	traits::{
		accessors::get::{GetField, GetFieldRef, Getter, GetterRef},
		handles_equipment::{CompatibleItems, ItemAsset},
		thread_safe::ThreadSafe,
	},
};

impl<T> VisualizeInvalidSkill for T {}

#[allow(clippy::type_complexity)]
pub(crate) trait VisualizeInvalidSkill {
	fn visualize_invalid_skill<TAgent, TSkill, TSlots>(
		mut commands: Commands,
		descriptors: Query<
			(Entity, &SkillButton<DropdownTrigger, TSkill>),
			Added<SkillButton<DropdownTrigger, TSkill>>,
		>,
		agents: Query<&TSlots, With<TAgent>>,
		items: Res<Assets<TSlots::TItem>>,
	) where
		Self: InsertContentOn + Sized,
		TSlots: Component + ItemAsset<TKey = SlotKey>,
		TSlots::TItem: Asset + Getter<ItemType>,
		TAgent: Component,
		TSkill: ThreadSafe + GetterRef<CompatibleItems>,
	{
		let Ok(agent) = agents.get_single() else {
			return;
		};

		let visualize = Self::insert_content_on;

		for descriptor in &descriptors {
			visualize_unusable(&mut commands, descriptor, agent, &items, visualize);
		}
	}
}

fn visualize_unusable<TSlots, TSkill>(
	commands: &mut Commands,
	(entity, descriptor): (Entity, &SkillButton<DropdownTrigger, TSkill>),
	slots: &TSlots,
	items: &Assets<TSlots::TItem>,
	visualize: fn(&mut EntityCommands),
) -> Option<()>
where
	TSlots: ItemAsset<TKey = SlotKey>,
	TSlots::TItem: Getter<ItemType>,
	TSkill: GetterRef<CompatibleItems>,
{
	let item = descriptor
		.key_path
		.last()
		.and_then(|key| slots.item_asset(key).ok())
		.and_then(|item| item.as_ref())
		.and_then(|item| items.get(item))?;

	let item_type = ItemType::get_field(item);
	let CompatibleItems(is_usable_with) = CompatibleItems::get_field_ref(&descriptor.skill);
	if is_usable_with.contains(&item_type) {
		return None;
	}

	let mut entity = commands.get_entity(entity)?;

	visualize(&mut entity);

	Some(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		test_tools::utils::SingleThreadedApp,
		tools::slot_key::Side,
		traits::{accessors::get::GetterRef, handles_equipment::KeyOutOfBounds},
	};
	use std::collections::{HashMap, HashSet};

	#[derive(Component)]
	struct _Agent;

	#[derive(Debug, PartialEq)]
	struct _Skill(CompatibleItems);

	impl GetterRef<CompatibleItems> for _Skill {
		fn get(&self) -> &CompatibleItems {
			&self.0
		}
	}

	#[derive(Asset, TypePath)]
	struct _Item(ItemType);

	impl Getter<ItemType> for _Item {
		fn get(&self) -> ItemType {
			self.0
		}
	}

	#[derive(Component)]
	struct _Slots(HashMap<SlotKey, Option<Handle<_Item>>>);

	impl<const N: usize> From<[(SlotKey, Option<Handle<_Item>>); N]> for _Slots {
		fn from(value: [(SlotKey, Option<Handle<_Item>>); N]) -> Self {
			Self(HashMap::from(value))
		}
	}

	impl ItemAsset for _Slots {
		type TKey = SlotKey;
		type TItem = _Item;

		fn item_asset(
			&self,
			key: &Self::TKey,
		) -> Result<&Option<Handle<Self::TItem>>, KeyOutOfBounds> {
			let Some(item) = self.0.get(key) else {
				return Err(KeyOutOfBounds);
			};

			Ok(item)
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Visualization;

	impl InsertContentOn for _Visualization {
		fn insert_content_on(entity: &mut EntityCommands) {
			entity.insert(_Visualization);
		}
	}

	fn setup_slots<const N: usize>(slots: [(SlotKey, _Item); N]) -> (_Slots, Assets<_Item>) {
		let mut items = Assets::default();
		let slots = slots
			.into_iter()
			.map(|(key, item)| (key, Some(items.add(item))))
			.collect();

		(_Slots(slots), items)
	}

	fn setup_app_and_slots_on<const N: usize>(
		agent: impl Component,
		slots: [(SlotKey, _Item); N],
	) -> App {
		let (slots, items) = setup_slots(slots);
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(items);
		app.world_mut().spawn((agent, slots));
		app.add_systems(
			Update,
			_Visualization::visualize_invalid_skill::<_Agent, _Skill, _Slots>,
		);

		app
	}

	#[test]
	fn visualize_unusable() {
		let mut app = setup_app_and_slots_on(
			_Agent,
			[(SlotKey::BottomHand(Side::Right), _Item(ItemType::Pistol))],
		);
		let skill = app
			.world_mut()
			.spawn(SkillButton::<DropdownTrigger, _Skill>::new(
				_Skill(CompatibleItems(HashSet::from([ItemType::Bracer]))),
				vec![
					SlotKey::BottomHand(Side::Left),
					SlotKey::BottomHand(Side::Right),
				],
			))
			.id();

		app.update();

		let skill = app.world().entity(skill);
		assert_eq!(Some(&_Visualization), skill.get::<_Visualization>())
	}

	#[test]
	fn do_not_visualize_usable() {
		let mut app = setup_app_and_slots_on(
			_Agent,
			[(SlotKey::BottomHand(Side::Right), _Item(ItemType::Pistol))],
		);
		let skill = app
			.world_mut()
			.spawn(SkillButton::<DropdownTrigger, _Skill>::new(
				_Skill(CompatibleItems(HashSet::from([ItemType::Pistol]))),
				vec![
					SlotKey::BottomHand(Side::Left),
					SlotKey::BottomHand(Side::Right),
				],
			))
			.id();

		app.update();

		let skill = app.world().entity(skill);
		assert_eq!(None, skill.get::<_Visualization>())
	}

	#[test]
	fn do_not_visualize_when_no_agents() {
		#[derive(Component)]
		struct _NonAgent;

		let mut app = setup_app_and_slots_on(
			_NonAgent,
			[(SlotKey::BottomHand(Side::Right), _Item(ItemType::Bracer))],
		);
		let skill = app
			.world_mut()
			.spawn(SkillButton::<DropdownTrigger, _Skill>::new(
				_Skill(CompatibleItems(HashSet::from([ItemType::Pistol]))),
				vec![
					SlotKey::BottomHand(Side::Left),
					SlotKey::BottomHand(Side::Right),
				],
			))
			.id();

		app.update();

		let skill = app.world().entity(skill);
		assert_eq!(None, skill.get::<_Visualization>())
	}

	#[test]
	fn do_not_visualize_when_not_added() {
		let mut app = setup_app_and_slots_on(
			_Agent,
			[(SlotKey::BottomHand(Side::Right), _Item(ItemType::Bracer))],
		);
		let skill = app
			.world_mut()
			.spawn(SkillButton::<DropdownTrigger, _Skill>::new(
				_Skill(CompatibleItems(HashSet::from([ItemType::Pistol]))),
				vec![
					SlotKey::BottomHand(Side::Left),
					SlotKey::BottomHand(Side::Right),
				],
			))
			.id();

		app.update();
		app.world_mut().entity_mut(skill).remove::<_Visualization>();
		app.update();

		let skill = app.world().entity(skill);
		assert_eq!(None, skill.get::<_Visualization>())
	}
}
