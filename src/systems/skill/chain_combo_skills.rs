use crate::{
	components::{Active, ComboTrees, ComboTreesRunning, SlotKey, Slots, Track, WaitNext},
	skill::{Skill, SkillComboTree},
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
	let tree_running = combo_trees_running.and_then(|t| t.0.get(&track.value.data.slot));
	let tree = combo_trees.0.get(&track.value.data.slot);
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
	for (slot_key, skill_combo_tree) in &tree.next {
		update_slot(slots, slot_key, &skill_combo_tree.skill);
	}
	agent.insert(ComboTreesRunning(tree.next.clone()));
}

fn update_slot(slots: &mut Mut<Slots>, slot_key: &SlotKey, skill: &Skill) {
	let Some(slot) = slots.0.get_mut(slot_key) else {
		return;
	};
	slot.combo_skill = Some(skill.clone());
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{Side, Slot, SlotKey},
		skill::SkillComboTree,
	};
	use bevy::{
		app::{App, Update},
		ecs::entity::Entity,
		utils::default,
	};
	use std::collections::HashMap;

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

	fn setup(running_skill_slot_key: SlotKey, combo_trees: ComboTrees) -> (App, Entity) {
		let track = Track::new(running_skill().with(Active {
			slot: running_skill_slot_key,
			..default()
		}));

		let mut app = App::new();
		let agent = app
			.world
			.spawn((combo_trees, track, Slots(HashMap::from(SLOTS.clone()))))
			.id();
		app.add_systems(Update, chain_combo_skills);

		(app, agent)
	}

	fn skill(name: &'static str) -> Skill {
		Skill { name, ..default() }
	}

	fn running_skill() -> Skill {
		Skill {
			name: "running skill",
			..default()
		}
	}

	#[test]
	fn set_slot_combo_skill_of_same_slot_key() {
		let running_skill_slot_key = SlotKey::Hand(Side::Off);
		let combo_trees = ComboTrees(HashMap::from([(
			running_skill_slot_key,
			SkillComboTree {
				skill: running_skill(),
				next: HashMap::from([(
					running_skill_slot_key,
					SkillComboTree {
						skill: skill("combo skill"),
						next: HashMap::from([]),
					},
				)]),
			},
		)]));
		let (mut app, agent) = setup(running_skill_slot_key, combo_trees);

		app.update();

		let agent = app.world.entity(agent);
		let slot = agent
			.get::<Slots>()
			.unwrap()
			.0
			.get(&running_skill_slot_key)
			.unwrap();

		assert_eq!(Some(skill("combo skill")), slot.combo_skill);
	}

	#[test]
	fn set_slot_combo_skill_of_other_slot_key() {
		let running_skill_slot_key = SlotKey::Hand(Side::Off);
		let other_slot_key = SlotKey::Hand(Side::Main);
		let combo_trees = ComboTrees(HashMap::from([(
			running_skill_slot_key,
			SkillComboTree {
				skill: running_skill(),
				next: HashMap::from([(
					other_slot_key,
					SkillComboTree {
						skill: skill("combo skill"),
						next: HashMap::from([]),
					},
				)]),
			},
		)]));
		let (mut app, agent) = setup(running_skill_slot_key, combo_trees);

		app.update();

		let agent = app.world.entity(agent);
		let slot = agent
			.get::<Slots>()
			.unwrap()
			.0
			.get(&other_slot_key)
			.unwrap();

		assert_eq!(Some(skill("combo skill")), slot.combo_skill);
	}

	#[test]
	fn do_not_set_slot_combo_when_running_skill_does_not_match_combo_skill() {
		let running_skill_slot_key = SlotKey::Hand(Side::Off);
		let combo_trees = ComboTrees(HashMap::from([(
			running_skill_slot_key,
			SkillComboTree {
				skill: skill("some random not running skill"),
				next: HashMap::from([(
					running_skill_slot_key,
					SkillComboTree {
						skill: skill("combo skill"),
						next: HashMap::from([]),
					},
				)]),
			},
		)]));
		let (mut app, agent) = setup(running_skill_slot_key, combo_trees);

		app.update();

		let agent = app.world.entity(agent);
		let slot = agent
			.get::<Slots>()
			.unwrap()
			.0
			.get(&running_skill_slot_key)
			.unwrap();

		assert_eq!(None, slot.combo_skill);
	}

	#[test]
	fn do_not_set_slot_combo_when_running_skill_does_not_match_combo_slot() {
		let running_skill_slot_key = SlotKey::Hand(Side::Main);
		let other_slot = SlotKey::Hand(Side::Off);
		let combo_trees = ComboTrees(HashMap::from([(
			other_slot,
			SkillComboTree {
				skill: running_skill(),
				next: HashMap::from([(
					running_skill_slot_key,
					SkillComboTree {
						skill: skill("combo skill"),
						next: HashMap::from([]),
					},
				)]),
			},
		)]));
		let (mut app, agent) = setup(running_skill_slot_key, combo_trees);

		app.update();

		let agent = app.world.entity(agent);
		let slot = agent
			.get::<Slots>()
			.unwrap()
			.0
			.get(&running_skill_slot_key)
			.unwrap();

		assert_eq!(None, slot.combo_skill);
	}

	#[test]
	fn set_slot_combo_skill_of_non_combo_slots_to_none() {
		let running_skill_slot_key = SlotKey::Hand(Side::Off);
		let combo_trees = ComboTrees(HashMap::from([(
			running_skill_slot_key,
			SkillComboTree {
				skill: running_skill(),
				next: HashMap::from([(
					running_skill_slot_key,
					SkillComboTree {
						skill: skill("combo skill"),
						next: HashMap::from([]),
					},
				)]),
			},
		)]));
		let (mut app, id) = setup(running_skill_slot_key, combo_trees);

		let mut agent = app.world.entity_mut(id);
		let mut slots = agent.get_mut::<Slots>().unwrap();
		for slot in slots.0.values_mut() {
			slot.combo_skill = Some(skill("fake combo skill"))
		}

		app.update();

		let agent = app.world.entity(id);
		let slots = agent.get::<Slots>().unwrap();
		let expected_slot_skills: HashMap<_, _> = SLOTS
			.clone()
			.into_iter()
			.map(|(key, _)| {
				if key == running_skill_slot_key {
					(key, Some(skill("combo skill")))
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
		let running_skill_slot_key = SlotKey::Hand(Side::Off);
		let combo_trees = ComboTrees(HashMap::from([(
			running_skill_slot_key,
			SkillComboTree {
				skill: skill("some not running skill"),
				next: HashMap::from([(
					running_skill_slot_key,
					SkillComboTree {
						skill: skill("combo skill"),
						next: HashMap::from([]),
					},
				)]),
			},
		)]));
		let (mut app, id) = setup(running_skill_slot_key, combo_trees);

		let mut agent = app.world.entity_mut(id);
		let mut slots = agent.get_mut::<Slots>().unwrap();
		for slot in slots.0.values_mut() {
			slot.combo_skill = Some(skill("fake combo skill"))
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
		let running_skill_slot_key = SlotKey::Hand(Side::Off);
		let mismatched_slot_key = SlotKey::Hand(Side::Main);
		let combo_trees = ComboTrees(HashMap::from([(
			mismatched_slot_key,
			SkillComboTree {
				skill: running_skill(),
				next: HashMap::from([(
					running_skill_slot_key,
					SkillComboTree {
						skill: skill("combo skill"),
						next: HashMap::from([]),
					},
				)]),
			},
		)]));
		let (mut app, id) = setup(running_skill_slot_key, combo_trees);

		let mut agent = app.world.entity_mut(id);
		let mut slots = agent.get_mut::<Slots>().unwrap();
		for slot in slots.0.values_mut() {
			slot.combo_skill = Some(skill("fake combo skill"))
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
		let (mut app, id) = setup(SlotKey::Hand(Side::Off), ComboTrees(HashMap::from([])));

		app.world.entity_mut(id).insert(WaitNext);
		app.world.entity_mut(id).remove::<Track<Skill<Active>>>();

		let mut agent = app.world.entity_mut(id);
		let mut slots = agent.get_mut::<Slots>().unwrap();
		for slot in slots.0.values_mut() {
			slot.combo_skill = Some(skill("fake combo skill"))
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
	fn update_combo_trees_running_to_tree_next() {
		let running_skill_slot_key = SlotKey::Hand(Side::Off);
		let next = HashMap::from([(
			running_skill_slot_key,
			SkillComboTree {
				skill: skill("combo skill"),
				next: HashMap::from([]),
			},
		)]);
		let combo_trees = ComboTrees(HashMap::from([]));
		let (mut app, agent) = setup(running_skill_slot_key, combo_trees);

		app.world
			.entity_mut(agent)
			.insert(ComboTreesRunning(HashMap::from([(
				running_skill_slot_key,
				SkillComboTree {
					skill: running_skill(),
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
		let running_skill_slot_key = SlotKey::Hand(Side::Off);
		let combo_trees = ComboTrees(HashMap::from([]));
		let (mut app, agent) = setup(running_skill_slot_key, combo_trees);

		app.world
			.entity_mut(agent)
			.insert(ComboTreesRunning(HashMap::from([(
				running_skill_slot_key,
				SkillComboTree {
					skill: running_skill(),
					next: HashMap::from([]),
				},
			)])));
		app.update();

		let agent = app.world.entity(agent);
		let combo_tree_running = agent.get::<ComboTreesRunning>();

		assert_eq!(None, combo_tree_running);
	}

	#[test]
	fn remove_combo_trees_running_when_skill_mismatch() {
		let running_skill_slot_key = SlotKey::Hand(Side::Off);
		let combo_trees = ComboTrees(HashMap::from([]));
		let (mut app, agent) = setup(running_skill_slot_key, combo_trees);

		app.world
			.entity_mut(agent)
			.insert(ComboTreesRunning(HashMap::from([(
				running_skill_slot_key,
				SkillComboTree {
					skill: skill("some not running"),
					next: HashMap::from([(
						running_skill_slot_key,
						SkillComboTree {
							skill: skill("combo skill"),
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
		let running_skill_slot_key = SlotKey::Hand(Side::Off);
		let mismatched_slot_key = SlotKey::Hand(Side::Main);
		let combo_trees = ComboTrees(HashMap::from([]));
		let (mut app, agent) = setup(running_skill_slot_key, combo_trees);

		app.world
			.entity_mut(agent)
			.insert(ComboTreesRunning(HashMap::from([(
				mismatched_slot_key,
				SkillComboTree {
					skill: running_skill(),
					next: HashMap::from([(
						running_skill_slot_key,
						SkillComboTree {
							skill: skill("combo skill"),
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
		let running_skill_slot_key = SlotKey::Hand(Side::Off);
		let next = HashMap::from([(
			running_skill_slot_key,
			SkillComboTree {
				skill: skill("combo skill"),
				next: HashMap::from([]),
			},
		)]);
		let combo_trees = ComboTrees(HashMap::from([(
			running_skill_slot_key,
			SkillComboTree {
				skill: running_skill(),
				next: next.clone(),
			},
		)]));
		let (mut app, agent) = setup(running_skill_slot_key, combo_trees);

		app.update();

		let agent = app.world.entity(agent);
		let combo_tree_running = agent.get::<ComboTreesRunning>();

		assert_eq!(Some(&ComboTreesRunning(next)), combo_tree_running);
	}

	#[test]
	fn use_running_if_present() {
		let running_skill_slot_key = SlotKey::Hand(Side::Off);
		let combo_trees = ComboTrees(HashMap::from([]));
		let (mut app, agent) = setup(running_skill_slot_key, combo_trees);

		app.world
			.entity_mut(agent)
			.insert(ComboTreesRunning(HashMap::from([(
				running_skill_slot_key,
				SkillComboTree {
					skill: running_skill(),
					next: HashMap::from([(
						running_skill_slot_key,
						SkillComboTree {
							skill: skill("combo skill"),
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
			.get(&running_skill_slot_key)
			.unwrap();

		assert_eq!(Some(skill("combo skill")), slot.combo_skill);
	}

	#[test]
	fn use_combo_tree_if_running_next_is_empty() {
		let running_skill_slot_key = SlotKey::Hand(Side::Off);
		let combo_trees = ComboTrees(HashMap::from([(
			running_skill_slot_key,
			SkillComboTree {
				skill: running_skill(),
				next: HashMap::from([(
					running_skill_slot_key,
					SkillComboTree {
						skill: skill("combo skill"),
						next: HashMap::from([]),
					},
				)]),
			},
		)]));
		let (mut app, agent) = setup(running_skill_slot_key, combo_trees);

		app.world
			.entity_mut(agent)
			.insert(ComboTreesRunning(HashMap::from([(
				running_skill_slot_key,
				SkillComboTree {
					skill: running_skill(),
					next: HashMap::from([]),
				},
			)])));
		app.update();

		let agent = app.world.entity(agent);
		let slot = agent
			.get::<Slots>()
			.unwrap()
			.0
			.get(&running_skill_slot_key)
			.unwrap();

		assert_eq!(Some(skill("combo skill")), slot.combo_skill);
	}
}
