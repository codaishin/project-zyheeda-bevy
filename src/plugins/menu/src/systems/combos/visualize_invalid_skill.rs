use crate::{
	components::skill_button::{DropdownTrigger, SkillButton},
	traits::InsertContentOn,
};
use bevy::{ecs::system::EntityCommands, prelude::*};
use common::{
	tools::{item_type::ItemType, slot_key::SlotKey},
	traits::{
		accessors::get::{GetField, Getter},
		handles_equipment::SingleAccess,
	},
};

impl<T> VisualizeInvalidSkill for T {}

pub(crate) trait VisualizeInvalidSkill {
	fn visualize_invalid_skill<TAgent, TVisualization>(
		mut commands: Commands,
		descriptors: Query<
			(Entity, &SkillButton<DropdownTrigger>),
			Added<SkillButton<DropdownTrigger>>,
		>,
		agents: Query<&Self, With<TAgent>>,
		items: Res<Assets<Self::TItem>>,
	) where
		Self: Component + SingleAccess<TKey = SlotKey> + Sized,
		Self::TItem: Asset + Getter<ItemType>,
		TAgent: Component,
		TVisualization: InsertContentOn,
	{
		let Ok(agent) = agents.get_single() else {
			return;
		};

		let visualize = TVisualization::insert_content_on;

		for descriptor in &descriptors {
			visualize_unusable(&mut commands, descriptor, agent, &items, visualize);
		}
	}
}

fn visualize_unusable<TSlots>(
	commands: &mut Commands,
	(entity, descriptor): (Entity, &SkillButton<DropdownTrigger>),
	slots: &TSlots,
	items: &Assets<TSlots::TItem>,
	visualize: fn(&mut EntityCommands),
) -> Option<()>
where
	TSlots: SingleAccess<TKey = SlotKey>,
	TSlots::TItem: Getter<ItemType>,
{
	let item = descriptor
		.key_path
		.last()
		.and_then(|key| slots.single_access(key))
		.and_then(|item| items.get(item))?;

	let item_type = ItemType::get_field(item);
	if descriptor.skill.is_usable_with.contains(&item_type) {
		return None;
	}

	let mut entity = commands.get_entity(entity)?;

	visualize(&mut entity);

	Some(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		ecs::system::EntityCommands,
		prelude::Component,
		utils::default,
	};
	use common::{test_tools::utils::SingleThreadedApp, tools::slot_key::Side};
	use skills::skills::Skill;
	use std::collections::{HashMap, HashSet};

	#[derive(Component)]
	struct _Agent;

	#[derive(Asset, TypePath)]
	struct _Item(ItemType);

	impl Getter<ItemType> for _Item {
		fn get(&self) -> ItemType {
			self.0
		}
	}

	#[derive(Component)]
	struct _Slots(HashMap<SlotKey, Handle<_Item>>);

	impl<const N: usize> From<[(SlotKey, Handle<_Item>); N]> for _Slots {
		fn from(value: [(SlotKey, Handle<_Item>); N]) -> Self {
			Self(HashMap::from(value))
		}
	}

	impl SingleAccess for _Slots {
		type TKey = SlotKey;
		type TItem = _Item;

		fn single_access(&self, key: &Self::TKey) -> Option<&Handle<Self::TItem>> {
			self.0.get(key)
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
			.map(|(key, item)| (key, items.add(item)))
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
			_Slots::visualize_invalid_skill::<_Agent, _Visualization>,
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
			.spawn(SkillButton::<DropdownTrigger>::new(
				Skill {
					is_usable_with: HashSet::from([ItemType::Bracer]),
					..default()
				},
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
			.spawn(SkillButton::<DropdownTrigger>::new(
				Skill {
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
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
			.spawn(SkillButton::<DropdownTrigger>::new(
				Skill {
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
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
			.spawn(SkillButton::<DropdownTrigger>::new(
				Skill {
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
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
