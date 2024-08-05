use crate::{
	components::skill_descriptor::{DropdownTrigger, SkillDescriptor},
	traits::InsertContentOn,
};
use bevy::{
	ecs::system::EntityCommands,
	prelude::{Added, Commands, Component, Entity, Query, With},
};
use common::traits::get::Get;
use skills::items::{slot_key::SlotKey, Item, ItemType};
use std::collections::HashSet;

pub(crate) fn visualize_invalid_skill<
	TAgent: Component,
	TSlots: Component + Get<SlotKey, Item>,
	TVisualization: InsertContentOn,
>(
	mut commands: Commands,
	descriptors: Query<
		(Entity, &SkillDescriptor<DropdownTrigger>),
		Added<SkillDescriptor<DropdownTrigger>>,
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

fn visualize_unusable<TSlots: Get<SlotKey, Item>>(
	commands: &mut Commands,
	(entity, descriptor): (Entity, &SkillDescriptor<DropdownTrigger>),
	agent: &TSlots,
	visualize: fn(&mut EntityCommands),
) -> Option<()> {
	let item = descriptor.key_path.last().and_then(|key| agent.get(key))?;

	if are_overlapping(&item.item_type, &descriptor.skill.is_usable_with) {
		return None;
	}

	let mut entity = commands.get_entity(entity)?;

	visualize(&mut entity);

	Some(())
}

fn are_overlapping(a: &HashSet<ItemType>, b: &HashSet<ItemType>) -> bool {
	a.intersection(b).next().is_some()
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
	use skills::{items::ItemType, skills::Skill};
	use std::collections::{HashMap, HashSet};

	#[derive(Component)]
	struct _Agent;

	#[derive(Component)]
	struct _Slots(HashMap<SlotKey, Item>);

	impl<const N: usize> From<[(SlotKey, Item); N]> for _Slots {
		fn from(value: [(SlotKey, Item); N]) -> Self {
			Self(HashMap::from(value))
		}
	}

	impl Get<SlotKey, Item> for _Slots {
		fn get<'a>(&'a self, key: &SlotKey) -> Option<&'a Item> {
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
				SlotKey::Hand(Side::Main),
				Item {
					item_type: HashSet::from([ItemType::Pistol]),
					..default()
				},
			)]),
		));
		let skill = app
			.world_mut()
			.spawn(SkillDescriptor::new_dropdown_trigger(
				Skill {
					is_usable_with: HashSet::from([ItemType::Bracer]),
					..default()
				},
				vec![SlotKey::Hand(Side::Off), SlotKey::Hand(Side::Main)],
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
				SlotKey::Hand(Side::Main),
				Item {
					item_type: HashSet::from([ItemType::Pistol]),
					..default()
				},
			)]),
		));
		let skill = app
			.world_mut()
			.spawn(SkillDescriptor::new_dropdown_trigger(
				Skill {
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
				vec![SlotKey::Hand(Side::Off), SlotKey::Hand(Side::Main)],
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
			SlotKey::Hand(Side::Main),
			Item {
				item_type: HashSet::from([ItemType::Bracer]),
				..default()
			},
		)]),));
		let skill = app
			.world_mut()
			.spawn(SkillDescriptor::new_dropdown_trigger(
				Skill {
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
				vec![SlotKey::Hand(Side::Off), SlotKey::Hand(Side::Main)],
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
				SlotKey::Hand(Side::Main),
				Item {
					item_type: HashSet::from([ItemType::Bracer]),
					..default()
				},
			)]),
		));
		let skill = app
			.world_mut()
			.spawn(SkillDescriptor::new_dropdown_trigger(
				Skill {
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
				vec![SlotKey::Hand(Side::Off), SlotKey::Hand(Side::Main)],
			))
			.id();

		app.update();

		app.world_mut().entity_mut(skill).remove::<_Visualization>();

		app.update();

		let skill = app.world().entity(skill);

		assert_eq!(None, skill.get::<_Visualization>())
	}
}
