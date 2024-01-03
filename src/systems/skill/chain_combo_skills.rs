use crate::{
	components::{ComboTrees, ComboTreesRunning, Item, SlotKey, Slots, Track, WaitNext},
	skill::{Active, Skill, SkillComboTree},
};
use bevy::{
	ecs::{
		query::{Added, Without},
		system::{Commands, EntityCommands, Query},
		world::Mut,
	},
	prelude::Entity,
};

type SkillComboComponents<'a> = (
	Entity,
	&'a ComboTrees,
	Option<&'a ComboTreesRunning>,
	&'a Track<Skill<Active>>,
	&'a mut Slots,
);

type JustStarted = (Added<Track<Skill<Active>>>, Without<WaitNext>);

pub fn chain_combo_skills(
	mut commands: Commands,
	mut idle: Query<(Entity, &mut Slots), Added<WaitNext>>,
	mut newly_active: Query<SkillComboComponents, JustStarted>,
) {
	for (agent, mut slots) in &mut idle {
		let agent = &mut commands.entity(agent);
		let slots = &mut slots;

		clear_combos(agent, slots);
	}

	for (agent, combo_trees, combo_trees_running, track, mut slots) in &mut newly_active {
		let agent = &mut commands.entity(agent);
		let slots = &mut slots;
		let tree = get_tree(combo_trees, combo_trees_running, track);

		clear_combos(agent, slots);

		if let Some(tree) = tree {
			update_combos(agent, slots, &tree);
		}
	}
}

fn get_tree(
	combo_trees: &ComboTrees,
	combo_trees_running: Option<&ComboTreesRunning>,
	track: &Track<Skill<Active>>,
) -> Option<SkillComboTree> {
	let tree_running = combo_trees_running.and_then(|t| t.0.get(&track.value.data.slot_key));
	let tree = combo_trees.0.get(&track.value.data.slot_key);
	let tree = match (tree_running, tree) {
		(Some(tree), _) if !tree.next.is_empty() => Some(tree),
		(_, Some(tree)) => Some(tree),
		_ => None,
	}?;

	if tree.skill.name != track.value.name {
		return None;
	}

	Some(tree.clone())
}

fn clear_combos(agent: &mut EntityCommands, slots: &mut Mut<Slots>) {
	for slot in &mut slots.0.values_mut() {
		slot.combo_skill = None;
	}
	agent.remove::<ComboTreesRunning>();
}

fn update_combos(agent: &mut EntityCommands, slots: &mut Mut<Slots>, tree: &SkillComboTree) {
	let remaining = tree
		.next
		.clone()
		.into_iter()
		.filter(|branch| update_slot(slots, branch));

	agent.insert(ComboTreesRunning(remaining.collect()));
}

fn update_slot(slots: &mut Mut<Slots>, (slot_key, tree): &(SlotKey, SkillComboTree)) -> bool {
	let Some(slot) = slots.0.get_mut(slot_key) else {
		return false;
	};

	let Some(item) = &slot.item else {
		return false;
	};

	if item_cannot_use_skill(item, &tree.skill) {
		return false;
	};

	slot.combo_skill = Some(tree.skill.clone());
	true
}

fn item_cannot_use_skill(item: &Item, skill: &Skill) -> bool {
	skill.is_usable_with.intersection(&item.item_type).count() == 0
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{Item, ItemType, Side, Slot, SlotKey},
		skill::SkillComboTree,
	};
	use bevy::{
		app::{App, Update},
		ecs::entity::Entity,
		utils::default,
	};
	use std::collections::{HashMap, HashSet};

	static SLOTS: [(SlotKey, Slot); 3] = [
		(
			SlotKey::Hand(Side::Off),
			Slot {
				entity: Entity::from_raw(42),
				item: None,
				combo_skill: None,
			},
		),
		(
			SlotKey::Hand(Side::Main),
			Slot {
				entity: Entity::from_raw(43),
				item: None,
				combo_skill: None,
			},
		),
		(
			SlotKey::Legs,
			Slot {
				entity: Entity::from_raw(44),
				item: None,
				combo_skill: None,
			},
		),
	];

	fn setup(
		running_skill: &Skill<Active>,
		combo_trees: &ComboTrees,
		item_types: &[(SlotKey, HashSet<ItemType>)],
	) -> (App, Entity) {
		let track = Track::new(running_skill.clone());
		let item_types: HashMap<SlotKey, HashSet<ItemType>> =
			HashMap::from_iter(item_types.iter().cloned());
		let slots = SLOTS.clone().map(|(key, mut slot)| {
			if let Some(item_type) = item_types.get(&key) {
				slot.item = Some(Item {
					item_type: item_type.clone(),
					..default()
				});
			}
			(key, slot)
		});

		let mut app = App::new();
		let agent = app
			.world
			.spawn((combo_trees.clone(), track, Slots(HashMap::from(slots))))
			.id();
		app.add_systems(Update, chain_combo_skills);

		(app, agent)
	}

	fn skill_usable_with<TItemTypes: ItemTypes>(types: TItemTypes) -> Skill {
		Skill {
			is_usable_with: types.to_set(),
			..default()
		}
	}

	trait ItemTypes {
		fn to_set(&self) -> HashSet<ItemType>;
	}

	impl<const N: usize> ItemTypes for &[ItemType; N] {
		fn to_set(&self) -> HashSet<ItemType> {
			HashSet::from_iter(self.iter().cloned())
		}
	}

	impl ItemTypes for &HashSet<ItemType> {
		fn to_set(&self) -> HashSet<ItemType> {
			HashSet::from_iter(self.iter().cloned())
		}
	}

	trait Named {
		fn named(&self, name: &'static str) -> Self;
	}

	impl Named for Skill {
		fn named(&self, name: &'static str) -> Self {
			let mut skill = self.clone();
			skill.name = name;
			skill
		}
	}

	trait ActiveOn {
		fn active_on(&self, slot_key: SlotKey) -> Skill<Active>;
	}

	impl ActiveOn for Skill {
		fn active_on(&self, slot_key: SlotKey) -> Skill<Active> {
			self.clone().with(&Active {
				slot_key,
				..default()
			})
		}
	}

	#[test]
	fn set_slot_combo_skill_of_same_slot_key() {
		let running = &skill_usable_with(&[ItemType::Sword])
			.named("combo-able")
			.active_on(SlotKey::Hand(Side::Off));
		let slots_item_types = &[(running.data.slot_key, running.is_usable_with.clone())];
		let combo_trees = &ComboTrees(HashMap::from([(
			running.data.slot_key,
			SkillComboTree {
				skill: skill_usable_with(&[]).named("combo-able"),
				next: HashMap::from([(
					running.data.slot_key,
					SkillComboTree {
						skill: skill_usable_with(&running.is_usable_with).named("combo skill"),
						next: HashMap::from([]),
					},
				)]),
			},
		)]));
		let (mut app, agent) = setup(running, combo_trees, slots_item_types);

		app.update();

		let agent = app.world.entity(agent);
		let slot = agent
			.get::<Slots>()
			.unwrap()
			.0
			.get(&running.data.slot_key)
			.unwrap();

		assert_eq!(
			Some(skill_usable_with(&running.is_usable_with).named("combo skill")),
			slot.combo_skill
		);
	}

	#[test]
	fn set_slot_combo_skill_of_other_slot_key() {
		let running = &skill_usable_with(&[ItemType::Sword])
			.named("combo-able")
			.active_on(SlotKey::Hand(Side::Off));
		let other_slot_key = SlotKey::Hand(Side::Main);
		let slots_item_types = &[
			(running.data.slot_key, running.is_usable_with.clone()),
			(other_slot_key, running.is_usable_with.clone()),
		];
		let combo_trees = &ComboTrees(HashMap::from([(
			running.data.slot_key,
			SkillComboTree {
				skill: skill_usable_with(&[]).named("combo-able"),
				next: HashMap::from([(
					other_slot_key,
					SkillComboTree {
						skill: skill_usable_with(&running.is_usable_with).named("combo skill"),
						next: HashMap::from([]),
					},
				)]),
			},
		)]));
		let (mut app, agent) = setup(running, combo_trees, slots_item_types);

		app.update();

		let agent = app.world.entity(agent);
		let slot = agent
			.get::<Slots>()
			.unwrap()
			.0
			.get(&other_slot_key)
			.unwrap();

		assert_eq!(
			Some(skill_usable_with(&running.is_usable_with).named("combo skill")),
			slot.combo_skill
		);
	}

	#[test]
	fn do_not_set_slot_combo_when_on_skill_name_mismatch() {
		let running = &skill_usable_with(&[ItemType::Sword])
			.named("combo-able")
			.active_on(SlotKey::Hand(Side::Off));
		let slots_item_types = &[(running.data.slot_key, running.is_usable_with.clone())];
		let combo_trees = &ComboTrees(HashMap::from([(
			running.data.slot_key,
			SkillComboTree {
				skill: skill_usable_with(&[]).named("not combo-able"),
				next: HashMap::from([(
					running.data.slot_key,
					SkillComboTree {
						skill: skill_usable_with(&running.is_usable_with).named("combo skill"),
						next: HashMap::from([]),
					},
				)]),
			},
		)]));
		let (mut app, agent) = setup(running, combo_trees, slots_item_types);

		app.update();

		let agent = app.world.entity(agent);
		let slot = agent
			.get::<Slots>()
			.unwrap()
			.0
			.get(&running.data.slot_key)
			.unwrap();

		assert_eq!(None, slot.combo_skill);
	}

	#[test]
	fn do_not_set_slot_combo_when_running_skill_does_not_match_combo_slot() {
		let running = &skill_usable_with(&[ItemType::Sword])
			.named("combo-able")
			.active_on(SlotKey::Hand(Side::Off));
		let slots_item_types = &[(running.data.slot_key, running.is_usable_with.clone())];
		let other_slot_key = SlotKey::Hand(Side::Main);
		let combo_trees = &ComboTrees(HashMap::from([(
			other_slot_key,
			SkillComboTree {
				skill: skill_usable_with(&[]).named("combo-able"),
				next: HashMap::from([(
					running.data.slot_key,
					SkillComboTree {
						skill: skill_usable_with(&running.is_usable_with).named("combo skill"),
						next: HashMap::from([]),
					},
				)]),
			},
		)]));
		let (mut app, agent) = setup(running, combo_trees, slots_item_types);

		app.update();

		let agent = app.world.entity(agent);
		let slot = agent
			.get::<Slots>()
			.unwrap()
			.0
			.get(&running.data.slot_key)
			.unwrap();

		assert_eq!(None, slot.combo_skill);
	}

	#[test]
	fn do_not_set_slot_combo_when_combo_skill_not_usable_by_slot_item() {
		let running = &skill_usable_with(&[ItemType::Sword])
			.named("combo-able")
			.active_on(SlotKey::Hand(Side::Off));
		let slots_item_types = &[(running.data.slot_key, HashSet::from([ItemType::Pistol]))];
		let combo_trees = &ComboTrees(HashMap::from([(
			running.data.slot_key,
			SkillComboTree {
				skill: skill_usable_with(&[]).named("combo-able"),
				next: HashMap::from([(
					running.data.slot_key,
					SkillComboTree {
						skill: skill_usable_with(&running.is_usable_with).named("combo skill"),
						next: HashMap::from([]),
					},
				)]),
			},
		)]));
		let (mut app, agent) = setup(running, combo_trees, slots_item_types);

		app.update();

		let agent = app.world.entity(agent);
		let slot = agent
			.get::<Slots>()
			.unwrap()
			.0
			.get(&running.data.slot_key)
			.unwrap();

		assert_eq!(None, slot.combo_skill);
	}

	#[test]
	fn set_slot_combo_when_combo_skill_usable_by_slot_item_via_intersection() {
		let running = &skill_usable_with(&[ItemType::Sword, ItemType::Legs])
			.named("combo-able")
			.active_on(SlotKey::Hand(Side::Off));
		let slots_item_types = &[(
			running.data.slot_key,
			HashSet::from([ItemType::Pistol, ItemType::Legs]),
		)];
		let combo_trees = &ComboTrees(HashMap::from([(
			running.data.slot_key,
			SkillComboTree {
				skill: skill_usable_with(&[]).named("combo-able"),
				next: HashMap::from([(
					running.data.slot_key,
					SkillComboTree {
						skill: skill_usable_with(&running.is_usable_with).named("combo skill"),
						next: HashMap::from([]),
					},
				)]),
			},
		)]));
		let (mut app, agent) = setup(running, combo_trees, slots_item_types);

		app.update();

		let agent = app.world.entity(agent);
		let slot = agent
			.get::<Slots>()
			.unwrap()
			.0
			.get(&running.data.slot_key)
			.unwrap();

		assert_eq!(
			Some(skill_usable_with(&running.is_usable_with).named("combo skill")),
			slot.combo_skill
		);
	}

	#[test]
	fn set_slot_combo_skill_of_non_combo_slots_to_none() {
		let running = &skill_usable_with(&[ItemType::Sword])
			.named("combo-able")
			.active_on(SlotKey::Hand(Side::Off));
		let combo_skill = &skill_usable_with(&running.is_usable_with).named("combo skill");
		let slots_item_types = &[(running.data.slot_key, running.is_usable_with.clone())];
		let combo_trees = &ComboTrees(HashMap::from([(
			running.data.slot_key,
			SkillComboTree {
				skill: skill_usable_with(&[]).named("combo-able"),
				next: HashMap::from([(
					running.data.slot_key,
					SkillComboTree {
						skill: combo_skill.clone(),
						next: HashMap::from([]),
					},
				)]),
			},
		)]));
		let (mut app, id) = setup(running, combo_trees, slots_item_types);

		let mut agent = app.world.entity_mut(id);
		let mut slots = agent.get_mut::<Slots>().unwrap();
		for slot in slots.0.values_mut() {
			slot.combo_skill = Some(skill_usable_with(&[]))
		}

		app.update();

		let agent = app.world.entity(id);
		let slots = agent.get::<Slots>().unwrap();
		let expected_slot_skills: HashMap<_, _> = SLOTS
			.clone()
			.into_iter()
			.map(|(key, _)| {
				if key == running.data.slot_key {
					(key, Some(combo_skill.clone()))
				} else {
					(key, None)
				}
			})
			.collect();
		let got_slot_skills: HashMap<_, _> = slots
			.0
			.clone()
			.into_iter()
			.map(|(k, s)| (k, s.combo_skill))
			.collect();

		assert_eq!(expected_slot_skills, got_slot_skills)
	}

	#[test]
	fn set_slot_combo_skill_to_none_on_name_mismatch() {
		let running = &skill_usable_with(&[ItemType::Sword])
			.named("combo-able")
			.active_on(SlotKey::Hand(Side::Off));
		let combo_skill = &skill_usable_with(&running.is_usable_with).named("combo skill");
		let slots_item_types = &[(running.data.slot_key, running.is_usable_with.clone())];
		let combo_trees = &ComboTrees(HashMap::from([(
			running.data.slot_key,
			SkillComboTree {
				skill: skill_usable_with(&[]).named("not combo-able"),
				next: HashMap::from([(
					running.data.slot_key,
					SkillComboTree {
						skill: combo_skill.clone(),
						next: HashMap::from([]),
					},
				)]),
			},
		)]));
		let (mut app, id) = setup(running, combo_trees, slots_item_types);

		let mut agent = app.world.entity_mut(id);
		let mut slots = agent.get_mut::<Slots>().unwrap();
		for slot in slots.0.values_mut() {
			slot.combo_skill = Some(skill_usable_with(&[]))
		}

		app.update();

		let agent = app.world.entity(id);
		let slots = agent.get::<Slots>().unwrap();
		let expected_slot_skills: HashMap<_, _> = SLOTS
			.clone()
			.into_iter()
			.map(|(key, _)| (key, None))
			.collect();
		let got_slot_skills: HashMap<_, _> = slots
			.0
			.clone()
			.into_iter()
			.map(|(k, s)| (k, s.combo_skill))
			.collect();

		assert_eq!(expected_slot_skills, got_slot_skills)
	}

	#[test]
	fn set_slot_combo_skill_to_none_on_slot_mismatch() {
		let running = &skill_usable_with(&[ItemType::Sword])
			.named("combo-able")
			.active_on(SlotKey::Hand(Side::Off));
		let combo_skill = &skill_usable_with(&running.is_usable_with).named("combo skill");
		let other_slot_key = SlotKey::Hand(Side::Main);
		let slots_item_types = &[(running.data.slot_key, running.is_usable_with.clone())];
		let combo_trees = &ComboTrees(HashMap::from([(
			other_slot_key,
			SkillComboTree {
				skill: skill_usable_with(&[]).named("combo-able"),
				next: HashMap::from([(
					running.data.slot_key,
					SkillComboTree {
						skill: combo_skill.clone(),
						next: HashMap::from([]),
					},
				)]),
			},
		)]));
		let (mut app, id) = setup(running, combo_trees, slots_item_types);

		let mut agent = app.world.entity_mut(id);
		let mut slots = agent.get_mut::<Slots>().unwrap();
		for slot in slots.0.values_mut() {
			slot.combo_skill = Some(skill_usable_with(&[]))
		}

		app.update();

		let agent = app.world.entity(id);
		let slots = agent.get::<Slots>().unwrap();
		let expected_slot_skills: HashMap<_, _> = SLOTS
			.clone()
			.into_iter()
			.map(|(key, _)| (key, None))
			.collect();
		let got_slot_skills: HashMap<_, _> = slots
			.0
			.clone()
			.into_iter()
			.map(|(k, s)| (k, s.combo_skill))
			.collect();

		assert_eq!(expected_slot_skills, got_slot_skills)
	}

	#[test]
	fn set_slot_combo_skill_to_none_when_waiting() {
		let running = &skill_usable_with(&[ItemType::Sword])
			.named("combo-able")
			.active_on(SlotKey::Hand(Side::Off));
		let slots_item_types = &[(running.data.slot_key, running.is_usable_with.clone())];
		let (mut app, id) = setup(running, &ComboTrees(HashMap::from([])), slots_item_types);

		app.world.entity_mut(id).insert(WaitNext);
		app.world.entity_mut(id).remove::<Track<Skill<Active>>>();

		let mut agent = app.world.entity_mut(id);
		let mut slots = agent.get_mut::<Slots>().unwrap();
		for slot in slots.0.values_mut() {
			slot.combo_skill = Some(skill_usable_with(&[]))
		}

		app.update();

		let agent = app.world.entity(id);
		let slots = agent.get::<Slots>().unwrap();
		let expected_slot_skills: HashMap<_, _> = SLOTS
			.clone()
			.into_iter()
			.map(|(key, _)| (key, None))
			.collect();
		let got_slot_skills: HashMap<_, _> = slots
			.0
			.clone()
			.into_iter()
			.map(|(k, s)| (k, s.combo_skill))
			.collect();

		assert_eq!(expected_slot_skills, got_slot_skills)
	}

	#[test]
	fn update_combo_trees_running_next() {
		let running = &skill_usable_with(&[ItemType::Sword])
			.named("combo-able")
			.active_on(SlotKey::Hand(Side::Off));
		let slots_item_types = &[(running.data.slot_key, running.is_usable_with.clone())];
		let next = HashMap::from([(
			running.data.slot_key,
			SkillComboTree {
				skill: skill_usable_with(&running.is_usable_with),
				next: HashMap::from([]),
			},
		)]);
		let combo_trees = &ComboTrees(HashMap::from([]));
		let (mut app, agent) = setup(running, combo_trees, slots_item_types);

		app.world
			.entity_mut(agent)
			.insert(ComboTreesRunning(HashMap::from([(
				running.data.slot_key,
				SkillComboTree {
					skill: skill_usable_with(&[]).named("combo-able"),
					next: next.clone(),
				},
			)])));
		app.update();

		let agent = app.world.entity(agent);
		let combo_tree_running = agent.get::<ComboTreesRunning>();

		assert_eq!(Some(&ComboTreesRunning(next)), combo_tree_running);
	}

	#[test]
	fn remove_combo_trees_running_when_next_empty() {
		let running = &skill_usable_with(&[ItemType::Sword])
			.named("combo-able")
			.active_on(SlotKey::Hand(Side::Off));
		let combo_trees = &ComboTrees(HashMap::from([]));
		let slots_item_types = &[(running.data.slot_key, running.is_usable_with.clone())];
		let (mut app, agent) = setup(running, combo_trees, slots_item_types);

		app.world
			.entity_mut(agent)
			.insert(ComboTreesRunning(HashMap::from([(
				running.data.slot_key,
				SkillComboTree {
					skill: skill_usable_with(&[]).named("combo-able"),
					next: HashMap::from([]),
				},
			)])));
		app.update();

		let agent = app.world.entity(agent);
		let combo_tree_running = agent.get::<ComboTreesRunning>();

		assert_eq!(None, combo_tree_running);
	}

	#[test]
	fn remove_combo_trees_running_on_name_mismatch() {
		let running = &skill_usable_with(&[ItemType::Sword])
			.named("combo-able")
			.active_on(SlotKey::Hand(Side::Off));
		let slots_item_types = &[(running.data.slot_key, running.is_usable_with.clone())];
		let combo_trees = &ComboTrees(HashMap::from([]));
		let (mut app, agent) = setup(running, combo_trees, slots_item_types);

		app.world
			.entity_mut(agent)
			.insert(ComboTreesRunning(HashMap::from([(
				running.data.slot_key,
				SkillComboTree {
					skill: skill_usable_with(&[]).named("not combo-able"),
					next: HashMap::from([(
						running.data.slot_key,
						SkillComboTree {
							skill: skill_usable_with(&running.is_usable_with).named("combo skill"),
							next: HashMap::from([]),
						},
					)]),
				},
			)])));
		app.update();

		let agent = app.world.entity(agent);
		let combo_tree_running = agent.get::<ComboTreesRunning>();

		assert_eq!(None, combo_tree_running);
	}

	#[test]
	fn remove_combo_trees_running_when_slot_mismatch() {
		let running = &skill_usable_with(&[ItemType::Sword])
			.named("combo-able")
			.active_on(SlotKey::Hand(Side::Off));
		let slots_item_types = &[(running.data.slot_key, running.is_usable_with.clone())];
		let mismatched_slot_key = SlotKey::Hand(Side::Main);
		let combo_trees = &ComboTrees(HashMap::from([]));
		let (mut app, agent) = setup(running, combo_trees, slots_item_types);

		app.world
			.entity_mut(agent)
			.insert(ComboTreesRunning(HashMap::from([(
				mismatched_slot_key,
				SkillComboTree {
					skill: skill_usable_with(&[]).named("combo-able"),
					next: HashMap::from([(
						running.data.slot_key,
						SkillComboTree {
							skill: skill_usable_with(&running.is_usable_with).named("combo skill"),
							next: HashMap::from([]),
						},
					)]),
				},
			)])));
		app.update();

		let agent = app.world.entity(agent);
		let combo_tree_running = agent.get::<ComboTreesRunning>();

		assert_eq!(None, combo_tree_running);
	}

	#[test]
	fn add_combo_trees_running_to_tree_next() {
		let running = &skill_usable_with(&[ItemType::Sword])
			.named("combo-able")
			.active_on(SlotKey::Hand(Side::Off));
		let slots_item_types = &[(running.data.slot_key, running.is_usable_with.clone())];
		let next = HashMap::from([(
			running.data.slot_key,
			SkillComboTree {
				skill: skill_usable_with(&running.is_usable_with).named("combo-able"),
				next: HashMap::from([]),
			},
		)]);
		let combo_trees = &ComboTrees(HashMap::from([(
			running.data.slot_key,
			SkillComboTree {
				skill: skill_usable_with(&running.is_usable_with).named("combo-able"),
				next: next.clone(),
			},
		)]));
		let (mut app, agent) = setup(running, combo_trees, slots_item_types);

		app.update();

		let agent = app.world.entity(agent);
		let combo_tree_running = agent.get::<ComboTreesRunning>();

		assert_eq!(Some(&ComboTreesRunning(next)), combo_tree_running);
	}

	#[test]
	fn add_item_usable_next_branches_to_running_next() {
		let running = &skill_usable_with(&[ItemType::Sword])
			.named("combo-able")
			.active_on(SlotKey::Hand(Side::Off));
		let other_slot_key = SlotKey::Hand(Side::Main);
		let slots_item_types = &[
			(running.data.slot_key, HashSet::from([ItemType::Sword])),
			(other_slot_key, HashSet::from([ItemType::Pistol])),
		];
		let combo_trees = &ComboTrees(HashMap::from([(
			running.data.slot_key,
			SkillComboTree {
				skill: skill_usable_with(&[]).named("combo-able"),
				next: HashMap::from([
					(
						running.data.slot_key,
						SkillComboTree {
							skill: skill_usable_with(&running.is_usable_with).named("combo skill"),
							next: HashMap::from([]),
						},
					),
					(
						other_slot_key,
						SkillComboTree {
							skill: skill_usable_with(&running.is_usable_with).named("combo skill"),
							next: HashMap::from([]),
						},
					),
				]),
			},
		)]));
		let (mut app, agent) = setup(running, combo_trees, slots_item_types);

		app.update();

		let agent = app.world.entity(agent);
		let combo_tree_running = agent.get::<ComboTreesRunning>();

		assert_eq!(
			Some(&ComboTreesRunning(HashMap::from([(
				running.data.slot_key,
				SkillComboTree {
					skill: skill_usable_with(&running.is_usable_with).named("combo skill"),
					next: HashMap::from([]),
				},
			)]))),
			combo_tree_running
		);
	}

	#[test]
	fn use_running_if_present() {
		let running = &skill_usable_with(&[ItemType::Sword])
			.named("combo-able")
			.active_on(SlotKey::Hand(Side::Off));
		let combo_skill = &skill_usable_with(&running.is_usable_with).named("combo skill");
		let combo_trees = &ComboTrees(HashMap::from([]));
		let slots_item_types = &[(running.data.slot_key, running.is_usable_with.clone())];
		let (mut app, agent) = setup(running, combo_trees, slots_item_types);

		app.world
			.entity_mut(agent)
			.insert(ComboTreesRunning(HashMap::from([(
				running.data.slot_key,
				SkillComboTree {
					skill: skill_usable_with(&[]).named("combo-able"),
					next: HashMap::from([(
						running.data.slot_key,
						SkillComboTree {
							skill: combo_skill.clone(),
							next: HashMap::from([]),
						},
					)]),
				},
			)])));
		app.update();

		let agent = app.world.entity(agent);
		let slot = agent
			.get::<Slots>()
			.unwrap()
			.0
			.get(&running.data.slot_key)
			.unwrap();

		assert_eq!(Some(combo_skill.clone()), slot.combo_skill);
	}

	#[test]
	fn use_combo_tree_if_running_next_is_empty() {
		let running = &skill_usable_with(&[ItemType::Sword])
			.named("combo-able")
			.active_on(SlotKey::Hand(Side::Off));
		let combo_skill = &skill_usable_with(&running.is_usable_with).named("combo skill");
		let slots_item_types = &[(running.data.slot_key, running.is_usable_with.clone())];
		let combo_trees = &ComboTrees(HashMap::from([(
			running.data.slot_key,
			SkillComboTree {
				skill: skill_usable_with(&[]).named("combo-able"),
				next: HashMap::from([(
					running.data.slot_key,
					SkillComboTree {
						skill: combo_skill.clone(),
						next: HashMap::from([]),
					},
				)]),
			},
		)]));
		let (mut app, agent) = setup(running, combo_trees, slots_item_types);

		app.world
			.entity_mut(agent)
			.insert(ComboTreesRunning(HashMap::from([(
				running.data.slot_key,
				SkillComboTree {
					skill: skill_usable_with(&[]).named("combo-able"),
					next: HashMap::from([]),
				},
			)])));
		app.update();

		let agent = app.world.entity(agent);
		let slot = agent
			.get::<Slots>()
			.unwrap()
			.0
			.get(&running.data.slot_key)
			.unwrap();

		assert_eq!(Some(combo_skill.clone()), slot.combo_skill);
	}
}
