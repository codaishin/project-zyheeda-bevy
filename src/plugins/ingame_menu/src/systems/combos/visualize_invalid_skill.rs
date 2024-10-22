use crate::{
	components::skill_button::{DropdownTrigger, SkillButton},
	traits::InsertContentOn,
};
use bevy::{
	ecs::system::EntityCommands,
	prelude::{Added, Commands, Component, Entity, Query, With},
};
use common::traits::accessors::get::GetRef;
use skills::{item::SkillItem, slot_key::SlotKey};

pub(crate) fn visualize_invalid_skill<
	TAgent: Component,
	TSlots: Component + GetRef<SlotKey, SkillItem>,
	TVisualization: InsertContentOn,
>(
	mut commands: Commands,
	descriptors: Query<
		(Entity, &SkillButton<DropdownTrigger>),
		Added<SkillButton<DropdownTrigger>>,
	>,
	agents: Query<&TSlots, With<TAgent>>,
) {
	let Ok(agent) = agents.get_single() else {
		return;
	};

	let visualize = TVisualization::insert_content_on;

	for descriptor in &descriptors {
		visualize_unusable(&mut commands, descriptor, agent, visualize);
	}
}

fn visualize_unusable<TSlots: GetRef<SlotKey, SkillItem>>(
	commands: &mut Commands,
	(entity, descriptor): (Entity, &SkillButton<DropdownTrigger>),
	agent: &TSlots,
	visualize: fn(&mut EntityCommands),
) -> Option<()> {
	let item = descriptor.key_path.last().and_then(|key| agent.get(key))?;

	if descriptor.skill.is_usable_with.contains(&item.item_type) {
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
	use common::{components::Side, test_tools::utils::SingleThreadedApp};
	use skills::{item::item_type::SkillItemType, skills::Skill};
	use std::collections::{HashMap, HashSet};

	#[derive(Component)]
	struct _Agent;

	#[derive(Component)]
	struct _Slots(HashMap<SlotKey, SkillItem>);

	impl<const N: usize> From<[(SlotKey, SkillItem); N]> for _Slots {
		fn from(value: [(SlotKey, SkillItem); N]) -> Self {
			Self(HashMap::from(value))
		}
	}

	impl GetRef<SlotKey, SkillItem> for _Slots {
		fn get<'a>(&'a self, key: &SlotKey) -> Option<&'a SkillItem> {
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

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			visualize_invalid_skill::<_Agent, _Slots, _Visualization>,
		);

		app
	}

	#[test]
	fn visualize_unusable() {
		let mut app = setup();
		app.world_mut().spawn((
			_Agent,
			_Slots::from([(
				SlotKey::BottomHand(Side::Right),
				SkillItem {
					item_type: SkillItemType::Pistol,
					..default()
				},
			)]),
		));
		let skill = app
			.world_mut()
			.spawn(SkillButton::<DropdownTrigger>::new(
				Skill {
					is_usable_with: HashSet::from([SkillItemType::Bracer]),
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
		let mut app = setup();
		app.world_mut().spawn((
			_Agent,
			_Slots::from([(
				SlotKey::BottomHand(Side::Right),
				SkillItem {
					item_type: SkillItemType::Pistol,
					..default()
				},
			)]),
		));
		let skill = app
			.world_mut()
			.spawn(SkillButton::<DropdownTrigger>::new(
				Skill {
					is_usable_with: HashSet::from([SkillItemType::Pistol]),
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
		let mut app = setup();
		app.world_mut().spawn((_Slots::from([(
			SlotKey::BottomHand(Side::Right),
			SkillItem {
				item_type: SkillItemType::Bracer,
				..default()
			},
		)]),));
		let skill = app
			.world_mut()
			.spawn(SkillButton::<DropdownTrigger>::new(
				Skill {
					is_usable_with: HashSet::from([SkillItemType::Pistol]),
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
		let mut app = setup();
		app.world_mut().spawn((
			_Agent,
			_Slots::from([(
				SlotKey::BottomHand(Side::Right),
				SkillItem {
					item_type: SkillItemType::Bracer,
					..default()
				},
			)]),
		));
		let skill = app
			.world_mut()
			.spawn(SkillButton::<DropdownTrigger>::new(
				Skill {
					is_usable_with: HashSet::from([SkillItemType::Pistol]),
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
