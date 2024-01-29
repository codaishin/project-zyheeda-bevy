use crate::{
	components::{
		ComboTreeRunning,
		ComboTreeTemplate,
		DequeueNext,
		Item,
		PlayerSkills,
		SideUnset,
		SlotKey,
		Slots,
		Track,
	},
	skill::{Active, Skill, SkillComboTree},
	traits::combo_next::ComboNext,
};
use bevy::{
	ecs::{
		query::{Added, Without},
		system::{Commands, EntityCommands, Query},
		world::Mut,
	},
	prelude::Entity,
};
use std::collections::HashMap;

type ComboComponents<'a, TNext> = (
	Entity,
	&'a Track<Skill<PlayerSkills<SideUnset>, Active>>,
	&'a ComboTreeTemplate<TNext>,
	Option<&'a ComboTreeRunning<TNext>>,
	&'a mut Slots,
);
type JustStarted = (
	Added<Track<Skill<PlayerSkills<SideUnset>, Active>>>,
	Without<DequeueNext>,
);
type Combos<TNext> = Vec<(SlotKey, SkillComboTree<TNext>)>;

pub fn chain_combo_skills<
	TNext: Clone + ComboNext<PlayerSkills<SideUnset>> + Send + Sync + 'static,
>(
	mut commands: Commands,
	mut newly_idle: Query<(Entity, &mut Slots), Added<DequeueNext>>,
	mut newly_active: Query<ComboComponents<TNext>, JustStarted>,
) {
	for (agent, mut slots) in &mut newly_idle {
		let agent = &mut commands.entity(agent);
		let slots = &mut slots;

		clear_combos::<TNext>(agent, slots);
	}

	for (agent, skill, combo_tree_t, combo_tree_r, mut slots) in &mut newly_active {
		let agent = &mut commands.entity(agent);
		let slots = &mut slots;

		clear_combos::<TNext>(agent, slots);
		update_combos(slots, agent, skill, combo_tree_t, combo_tree_r);
	}
}

fn update_combos<TNext: Clone + ComboNext<PlayerSkills<SideUnset>> + Send + Sync + 'static>(
	slots: &mut Mut<Slots>,
	agent: &mut EntityCommands,
	skill: &Track<Skill<PlayerSkills<SideUnset>, Active>>,
	combo_tree_t: &ComboTreeTemplate<TNext>,
	combo_tree_r: Option<&ComboTreeRunning<TNext>>,
) {
	let Some(combos) = get_combos(&skill.value, combo_tree_t, combo_tree_r) else {
		return;
	};

	let updated_successfully: HashMap<_, _> = combos
		.clone()
		.into_iter()
		.filter(|(slot_key, tree)| update_slot(slots, slot_key, &tree.skill))
		.collect();

	if updated_successfully.is_empty() {
		return;
	}

	agent.insert(ComboTreeRunning(updated_successfully));
}

fn get_combos<'a, TNext: ComboNext<PlayerSkills<SideUnset>>>(
	skill: &'a Skill<PlayerSkills<SideUnset>, Active>,
	combo_tree_t: &'a ComboTreeTemplate<TNext>,
	combo_tree_r: Option<&'a ComboTreeRunning<TNext>>,
) -> Option<Combos<TNext>> {
	let combos_r = combo_tree_r
		.and_then(|tree| tree.0.get(&skill.data.slot_key))
		.map(|combo| (&combo.skill, combo.next.to_vec(skill)))
		.filter(|(_, combo)| !combo.is_empty());

	let (combo_trigger, combos) = match combos_r {
		Some((combo_trigger, combos)) => (combo_trigger, combos),
		None => {
			let combo = combo_tree_t.0.get(&skill.data.slot_key)?;
			(&combo.skill, combo.next.to_vec(skill))
		}
	};

	if skill.name != combo_trigger.name {
		return None;
	}

	Some(combos)
}

fn clear_combos<TNext: Sync + Send + 'static>(agent: &mut EntityCommands, slots: &mut Mut<Slots>) {
	for slot in &mut slots.0.values_mut() {
		slot.combo_skill = None;
	}
	agent.remove::<ComboTreeRunning<TNext>>();
}

fn update_slot(slots: &mut Mut<Slots>, slot_key: &SlotKey, skill: &Skill) -> bool {
	let Some(slot) = slots.0.get_mut(slot_key) else {
		return false;
	};

	let Some(item) = &slot.item else {
		return false;
	};

	if item_cannot_use_skill(item, skill) {
		return false;
	};

	slot.combo_skill = Some(skill.clone());
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
	use mockall::{mock, predicate::eq};
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

	fn setup<TNext: Clone + ComboNext<PlayerSkills<SideUnset>> + Sync + Send + 'static>(
		running_skill: &Skill<PlayerSkills<SideUnset>, Active>,
		combo_tree: ComboTreeTemplate<TNext>,
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
			.spawn((combo_tree, track, Slots(HashMap::from(slots))))
			.id();
		app.add_systems(Update, chain_combo_skills::<TNext>);

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
		fn active_on(&self, slot_key: SlotKey) -> Skill<PlayerSkills<SideUnset>, Active>;
	}

	impl ActiveOn for Skill {
		fn active_on(&self, slot_key: SlotKey) -> Skill<PlayerSkills<SideUnset>, Active> {
			self.clone().with(&Active {
				slot_key,
				..default()
			})
		}
	}

	#[derive(PartialEq, Debug, Clone)]
	struct _Next(Vec<(SlotKey, SkillComboTree<_Next>)>);

	mock! {
		_Next{}
		impl ComboNext<PlayerSkills<SideUnset>> for _Next {
			fn to_vec(&self, _skill: &Skill<PlayerSkills<SideUnset>,Active>) -> Vec<(SlotKey, SkillComboTree<Self>)> {
				self.0.clone()
			}
		}
		impl Clone for _Next {
			fn clone(&self) -> Self {
				Self(self.0.clone())
			}
		}
	}

	impl ComboNext<PlayerSkills<SideUnset>> for _Next {
		fn to_vec(
			&self,
			_skill: &Skill<PlayerSkills<SideUnset>, Active>,
		) -> Vec<(SlotKey, SkillComboTree<Self>)> {
			self.0.clone()
		}
	}

	#[test]
	fn set_slot_combo_skill_of_same_slot_key() {
		let skill = &skill_usable_with(&[ItemType::Sword])
			.named("combo-start")
			.active_on(SlotKey::Hand(Side::Off));
		let slots_item_types = &[(skill.data.slot_key, skill.is_usable_with.clone())];
		let combo_tree = ComboTreeTemplate(HashMap::from([(
			skill.data.slot_key,
			SkillComboTree {
				skill: skill_usable_with(&[]).named("combo-start"),
				next: _Next(vec![(
					skill.data.slot_key,
					SkillComboTree {
						skill: skill_usable_with(&skill.is_usable_with).named("combo skill"),
						next: _Next(vec![]),
					},
				)]),
			},
		)]));

		let (mut app, agent) = setup(skill, combo_tree, slots_item_types);
		app.update();

		let agent = app.world.entity(agent);
		let slot = agent
			.get::<Slots>()
			.unwrap()
			.0
			.get(&skill.data.slot_key)
			.unwrap();

		assert_eq!(
			Some(skill_usable_with(&skill.is_usable_with).named("combo skill")),
			slot.combo_skill
		);
	}

	#[test]
	fn set_slot_combo_skill_of_other_slot_key() {
		let skill = &skill_usable_with(&[ItemType::Sword])
			.named("combo-start")
			.active_on(SlotKey::Hand(Side::Off));
		let other_slot_key = SlotKey::Hand(Side::Main);
		let slots_item_types = &[
			(skill.data.slot_key, skill.is_usable_with.clone()),
			(other_slot_key, skill.is_usable_with.clone()),
		];
		let combo_tree = ComboTreeTemplate(HashMap::from([(
			skill.data.slot_key,
			SkillComboTree {
				skill: skill_usable_with(&[]).named("combo-start"),
				next: _Next(vec![(
					other_slot_key,
					SkillComboTree {
						skill: skill_usable_with(&skill.is_usable_with).named("combo skill"),
						next: _Next(vec![]),
					},
				)]),
			},
		)]));

		let (mut app, agent) = setup(skill, combo_tree, slots_item_types);
		app.update();

		let agent = app.world.entity(agent);
		let slot = agent
			.get::<Slots>()
			.unwrap()
			.0
			.get(&other_slot_key)
			.unwrap();

		assert_eq!(
			Some(skill_usable_with(&skill.is_usable_with).named("combo skill")),
			slot.combo_skill
		);
	}

	#[test]
	fn do_not_set_slot_combo_when_on_skill_name_mismatch() {
		let skill = &skill_usable_with(&[ItemType::Sword])
			.named("combo-start")
			.active_on(SlotKey::Hand(Side::Off));
		let slots_item_types = &[(skill.data.slot_key, skill.is_usable_with.clone())];
		let combo_tree = ComboTreeTemplate(HashMap::from([(
			skill.data.slot_key,
			SkillComboTree {
				skill: skill_usable_with(&[]).named("not combo-start"),
				next: _Next(vec![(
					skill.data.slot_key,
					SkillComboTree {
						skill: skill_usable_with(&skill.is_usable_with).named("combo skill"),
						next: _Next(vec![]),
					},
				)]),
			},
		)]));

		let (mut app, agent) = setup(skill, combo_tree, slots_item_types);
		app.update();

		let agent = app.world.entity(agent);
		let slot = agent
			.get::<Slots>()
			.unwrap()
			.0
			.get(&skill.data.slot_key)
			.unwrap();

		assert_eq!(None, slot.combo_skill);
	}

	#[test]
	fn do_not_set_slot_combo_when_running_skill_does_not_match_combo_slot() {
		let skill = &skill_usable_with(&[ItemType::Sword])
			.named("combo-start")
			.active_on(SlotKey::Hand(Side::Off));
		let slots_item_types = &[(skill.data.slot_key, skill.is_usable_with.clone())];
		let other_slot_key = SlotKey::Hand(Side::Main);
		let combo_tree = ComboTreeTemplate(HashMap::from([(
			other_slot_key,
			SkillComboTree {
				skill: skill_usable_with(&[]).named("combo-start"),
				next: _Next(vec![(
					skill.data.slot_key,
					SkillComboTree {
						skill: skill_usable_with(&skill.is_usable_with).named("combo skill"),
						next: _Next(vec![]),
					},
				)]),
			},
		)]));

		let (mut app, agent) = setup(skill, combo_tree, slots_item_types);
		app.update();

		let agent = app.world.entity(agent);
		let slot = agent
			.get::<Slots>()
			.unwrap()
			.0
			.get(&skill.data.slot_key)
			.unwrap();

		assert_eq!(None, slot.combo_skill);
	}

	#[test]
	fn do_not_set_slot_combo_when_combo_skill_not_usable_by_slot_item() {
		let skill = &skill_usable_with(&[ItemType::Sword])
			.named("combo-start")
			.active_on(SlotKey::Hand(Side::Off));
		let slots_item_types = &[(skill.data.slot_key, HashSet::from([ItemType::Pistol]))];
		let combo_tree = ComboTreeTemplate(HashMap::from([(
			skill.data.slot_key,
			SkillComboTree {
				skill: skill_usable_with(&[]).named("combo-start"),
				next: _Next(vec![(
					skill.data.slot_key,
					SkillComboTree {
						skill: skill_usable_with(&skill.is_usable_with).named("combo skill"),
						next: _Next(vec![]),
					},
				)]),
			},
		)]));

		let (mut app, agent) = setup(skill, combo_tree, slots_item_types);
		app.update();

		let agent = app.world.entity(agent);
		let slot = agent
			.get::<Slots>()
			.unwrap()
			.0
			.get(&skill.data.slot_key)
			.unwrap();

		assert_eq!(None, slot.combo_skill);
	}

	#[test]
	fn set_slot_combo_when_combo_skill_usable_by_slot_item_via_intersection() {
		let skill = &skill_usable_with(&[ItemType::Sword, ItemType::Legs])
			.named("combo-start")
			.active_on(SlotKey::Hand(Side::Off));
		let slots_item_types = &[(
			skill.data.slot_key,
			HashSet::from([ItemType::Pistol, ItemType::Legs]),
		)];
		let combo_tree = ComboTreeTemplate(HashMap::from([(
			skill.data.slot_key,
			SkillComboTree {
				skill: skill_usable_with(&[]).named("combo-start"),
				next: _Next(vec![(
					skill.data.slot_key,
					SkillComboTree {
						skill: skill_usable_with(&skill.is_usable_with).named("combo skill"),
						next: _Next(vec![]),
					},
				)]),
			},
		)]));

		let (mut app, agent) = setup(skill, combo_tree, slots_item_types);
		app.update();

		let agent = app.world.entity(agent);
		let slot = agent
			.get::<Slots>()
			.unwrap()
			.0
			.get(&skill.data.slot_key)
			.unwrap();

		assert_eq!(
			Some(skill_usable_with(&skill.is_usable_with).named("combo skill")),
			slot.combo_skill
		);
	}

	#[test]
	fn set_slot_combo_skill_of_non_combo_slots_to_none() {
		let skill = &skill_usable_with(&[ItemType::Sword])
			.named("combo-start")
			.active_on(SlotKey::Hand(Side::Off));
		let combo_skill = &skill_usable_with(&skill.is_usable_with).named("combo skill");
		let slots_item_types = &[(skill.data.slot_key, skill.is_usable_with.clone())];
		let combo_tree = ComboTreeTemplate(HashMap::from([(
			skill.data.slot_key,
			SkillComboTree {
				skill: skill_usable_with(&[]).named("combo-start"),
				next: _Next(vec![(
					skill.data.slot_key,
					SkillComboTree {
						skill: combo_skill.clone(),
						next: _Next(vec![]),
					},
				)]),
			},
		)]));

		let (mut app, id) = setup(skill, combo_tree, slots_item_types);
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
				if key == skill.data.slot_key {
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
			.map(|(key, slot)| (key, slot.combo_skill))
			.collect();

		assert_eq!(expected_slot_skills, got_slot_skills)
	}

	#[test]
	fn set_slot_combo_skill_to_none_on_name_mismatch() {
		let skill = &skill_usable_with(&[ItemType::Sword])
			.named("combo-start")
			.active_on(SlotKey::Hand(Side::Off));
		let slots_item_types = &[(skill.data.slot_key, skill.is_usable_with.clone())];
		let combo_tree = ComboTreeTemplate(HashMap::from([(
			skill.data.slot_key,
			SkillComboTree {
				skill: skill_usable_with(&[]).named("not combo-start"),
				next: _Next(vec![(
					skill.data.slot_key,
					SkillComboTree {
						skill: skill_usable_with(&skill.is_usable_with).named("combo skill"),
						next: _Next(vec![]),
					},
				)]),
			},
		)]));

		let (mut app, id) = setup(skill, combo_tree, slots_item_types);
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
			.map(|(key, slot)| (key, slot.combo_skill))
			.collect();

		assert_eq!(expected_slot_skills, got_slot_skills)
	}

	#[test]
	fn set_slot_combo_skill_to_none_on_slot_mismatch() {
		let skill = &skill_usable_with(&[ItemType::Sword])
			.named("combo-start")
			.active_on(SlotKey::Hand(Side::Off));
		let other_slot_key = SlotKey::Hand(Side::Main);
		let slots_item_types = &[(skill.data.slot_key, skill.is_usable_with.clone())];
		let combo_tree = ComboTreeTemplate(HashMap::from([(
			other_slot_key,
			SkillComboTree {
				skill: skill_usable_with(&[]).named("combo-start"),
				next: _Next(vec![(
					skill.data.slot_key,
					SkillComboTree {
						skill: skill_usable_with(&skill.is_usable_with).named("combo skill"),
						next: _Next(vec![]),
					},
				)]),
			},
		)]));

		let (mut app, id) = setup(skill, combo_tree, slots_item_types);
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
			.map(|(key, slot)| (key, slot.combo_skill))
			.collect();

		assert_eq!(expected_slot_skills, got_slot_skills)
	}

	#[test]
	fn set_slot_combo_skill_to_none_when_waiting() {
		let skill = &skill_usable_with(&[ItemType::Sword])
			.named("combo-start")
			.active_on(SlotKey::Hand(Side::Off));
		let slots_item_types = &[(skill.data.slot_key, skill.is_usable_with.clone())];

		let (mut app, id) = setup::<_Next>(
			skill,
			ComboTreeTemplate(HashMap::from([])),
			slots_item_types,
		);
		let mut agent = app.world.entity_mut(id);
		let mut slots = agent.get_mut::<Slots>().unwrap();
		for slot in slots.0.values_mut() {
			slot.combo_skill = Some(skill_usable_with(&[]))
		}
		app.world.entity_mut(id).insert(DequeueNext);
		app.world
			.entity_mut(id)
			.remove::<Track<Skill<PlayerSkills<SideUnset>, Active>>>();
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
			.map(|(key, slot)| (key, slot.combo_skill))
			.collect();

		assert_eq!(expected_slot_skills, got_slot_skills)
	}

	#[test]
	fn update_combo_tree_running_next() {
		let skill = &skill_usable_with(&[ItemType::Sword])
			.named("combo-start")
			.active_on(SlotKey::Hand(Side::Off));
		let slots_item_types = &[(skill.data.slot_key, skill.is_usable_with.clone())];
		let next_values = [(
			skill.data.slot_key,
			SkillComboTree {
				skill: skill_usable_with(&skill.is_usable_with).named("combo skill"),
				next: _Next(vec![]),
			},
		)];
		let combo_tree = ComboTreeTemplate(HashMap::from([(
			skill.data.slot_key,
			SkillComboTree {
				skill: skill_usable_with(&[]).named("combo-start"),
				next: _Next(next_values.to_vec()),
			},
		)]));

		let (mut app, agent) = setup(skill, combo_tree, slots_item_types);
		app.world
			.entity_mut(agent)
			.insert(ComboTreeRunning(HashMap::from([(
				skill.data.slot_key,
				SkillComboTree {
					skill: skill_usable_with(&[]).named("other combo skill"),
					next: _Next(vec![]),
				},
			)])));
		app.update();

		let agent = app.world.entity(agent);
		let combo_tree_running = agent.get::<ComboTreeRunning<_Next>>();

		assert_eq!(
			Some(&ComboTreeRunning(HashMap::from(next_values))),
			combo_tree_running
		);
	}

	#[test]
	fn add_combo_tree_running_with_next() {
		let skill = &skill_usable_with(&[ItemType::Sword])
			.named("combo-start")
			.active_on(SlotKey::Hand(Side::Off));
		let slots_item_types = &[(skill.data.slot_key, skill.is_usable_with.clone())];
		let next_values = [(
			skill.data.slot_key,
			SkillComboTree {
				skill: skill_usable_with(&skill.is_usable_with).named("combo skill"),
				next: _Next(vec![]),
			},
		)];
		let combo_tree = ComboTreeTemplate(HashMap::from([(
			skill.data.slot_key,
			SkillComboTree {
				skill: skill_usable_with(&[]).named("combo-start"),
				next: _Next(next_values.to_vec()),
			},
		)]));

		let (mut app, agent) = setup(skill, combo_tree, slots_item_types);
		app.update();

		let agent = app.world.entity(agent);
		let combo_tree_running = agent.get::<ComboTreeRunning<_Next>>();

		assert_eq!(
			Some(&ComboTreeRunning(HashMap::from(next_values))),
			combo_tree_running
		);
	}

	#[test]
	fn remove_combo_tree_running_when_next_empty() {
		let skill = &skill_usable_with(&[ItemType::Sword])
			.named("combo-start")
			.active_on(SlotKey::Hand(Side::Off));
		let combo_tree = ComboTreeTemplate::<_Next>(HashMap::from([]));
		let slots_item_types = &[(skill.data.slot_key, skill.is_usable_with.clone())];

		let (mut app, agent) = setup(skill, combo_tree, slots_item_types);
		app.world
			.entity_mut(agent)
			.insert(ComboTreeRunning(HashMap::from([(
				skill.data.slot_key,
				SkillComboTree {
					skill: skill_usable_with(&[]).named("combo-start"),
					next: _Next(vec![]),
				},
			)])));
		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<ComboTreeRunning<_Next>>());
	}

	#[test]
	fn remove_combo_tree_running_on_name_mismatch() {
		let skill = &skill_usable_with(&[ItemType::Sword])
			.named("combo-start")
			.active_on(SlotKey::Hand(Side::Off));
		let slots_item_types = &[(skill.data.slot_key, skill.is_usable_with.clone())];
		let combo_tree = ComboTreeTemplate::<_Next>(HashMap::from([]));

		let (mut app, agent) = setup(skill, combo_tree, slots_item_types);
		app.world
			.entity_mut(agent)
			.insert(ComboTreeRunning(HashMap::from([(
				skill.data.slot_key,
				SkillComboTree {
					skill: skill_usable_with(&[]).named("not combo-start"),
					next: _Next(vec![(
						skill.data.slot_key,
						SkillComboTree {
							skill: skill_usable_with(&skill.is_usable_with).named("combo skill"),
							next: _Next(vec![]),
						},
					)]),
				},
			)])));
		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<ComboTreeRunning<_Next>>());
	}

	#[test]
	fn remove_combo_tree_running_when_slot_mismatch() {
		let skill = &skill_usable_with(&[ItemType::Sword])
			.named("combo-start")
			.active_on(SlotKey::Hand(Side::Off));
		let slots_item_types = &[(skill.data.slot_key, skill.is_usable_with.clone())];
		let mismatched_slot_key = SlotKey::Hand(Side::Main);
		let combo_tree = ComboTreeTemplate::<_Next>(HashMap::from([]));

		let (mut app, agent) = setup(skill, combo_tree, slots_item_types);
		app.world
			.entity_mut(agent)
			.insert(ComboTreeRunning(HashMap::from([(
				mismatched_slot_key,
				SkillComboTree {
					skill: skill_usable_with(&[]).named("combo-start"),
					next: _Next(vec![(
						skill.data.slot_key,
						SkillComboTree {
							skill: skill_usable_with(&skill.is_usable_with).named("combo skill"),
							next: _Next(vec![]),
						},
					)]),
				},
			)])));
		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<ComboTreeRunning<_Next>>());
	}

	#[test]
	fn add_only_valid_combos_to_running_next() {
		let skill = &skill_usable_with(&[ItemType::Sword])
			.named("combo-start")
			.active_on(SlotKey::Hand(Side::Off));
		let other_slot_key = SlotKey::Hand(Side::Main);
		let unusable_type = HashSet::from([ItemType::Pistol]);
		let slots_item_types = &[
			(skill.data.slot_key, skill.is_usable_with.clone()),
			(other_slot_key, skill.is_usable_with.clone()),
		];
		let combo_tree = ComboTreeTemplate(HashMap::from([(
			skill.data.slot_key,
			SkillComboTree {
				skill: skill_usable_with(&[]).named("combo-start"),
				next: _Next(vec![
					(
						skill.data.slot_key,
						SkillComboTree {
							skill: skill_usable_with(&skill.is_usable_with).named("combo skill"),
							next: _Next(vec![]),
						},
					),
					(
						other_slot_key,
						SkillComboTree {
							skill: skill_usable_with(&unusable_type).named("unusable combo skill"),
							next: _Next(vec![]),
						},
					),
				]),
			},
		)]));

		let (mut app, agent) = setup(skill, combo_tree, slots_item_types);
		app.update();

		let agent = app.world.entity(agent);
		let combo_tree_running = agent.get::<ComboTreeRunning<_Next>>();

		assert_eq!(
			Some(&ComboTreeRunning(HashMap::from([(
				skill.data.slot_key,
				SkillComboTree {
					skill: skill_usable_with(&skill.is_usable_with).named("combo skill"),
					next: _Next(vec![]),
				},
			)]))),
			combo_tree_running
		);
	}

	#[test]
	fn remove_running_when_no_combo_valid() {
		let skill = &skill_usable_with(&[ItemType::Sword])
			.named("combo-start")
			.active_on(SlotKey::Hand(Side::Off));
		let unusable_type = HashSet::from([ItemType::Pistol]);
		let slots_item_types = &[(skill.data.slot_key, skill.is_usable_with.clone())];
		let combo_tree = ComboTreeTemplate(HashMap::from([(
			skill.data.slot_key,
			SkillComboTree {
				skill: skill_usable_with(&[]).named("combo-start"),
				next: _Next(vec![(
					skill.data.slot_key,
					SkillComboTree {
						skill: skill_usable_with(&unusable_type).named("combo skill"),
						next: _Next(vec![]),
					},
				)]),
			},
		)]));

		let (mut app, agent) = setup(skill, combo_tree, slots_item_types);
		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<ComboTreeRunning<_Next>>());
	}

	#[test]
	fn use_running_if_present() {
		let skill = &skill_usable_with(&[ItemType::Sword])
			.named("combo-start")
			.active_on(SlotKey::Hand(Side::Off));
		let combo_skill = &skill_usable_with(&skill.is_usable_with).named("combo skill");
		let combo_tree = ComboTreeTemplate::<_Next>(HashMap::from([]));
		let slots_item_types = &[(skill.data.slot_key, skill.is_usable_with.clone())];
		let (mut app, agent) = setup(skill, combo_tree, slots_item_types);

		app.world
			.entity_mut(agent)
			.insert(ComboTreeRunning(HashMap::from([(
				skill.data.slot_key,
				SkillComboTree {
					skill: skill_usable_with(&[]).named("combo-start"),
					next: _Next(vec![(
						skill.data.slot_key,
						SkillComboTree {
							skill: combo_skill.clone(),
							next: _Next(vec![]),
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
			.get(&skill.data.slot_key)
			.unwrap();

		assert_eq!(Some(combo_skill.clone()), slot.combo_skill);
	}

	#[test]
	fn use_combo_tree_if_running_next_is_empty() {
		let skill = &skill_usable_with(&[ItemType::Sword])
			.named("combo-start")
			.active_on(SlotKey::Hand(Side::Off));
		let combo_skill = &skill_usable_with(&skill.is_usable_with).named("combo skill");
		let slots_item_types = &[(skill.data.slot_key, skill.is_usable_with.clone())];
		let combo_tree = ComboTreeTemplate(HashMap::from([(
			skill.data.slot_key,
			SkillComboTree {
				skill: skill_usable_with(&[]).named("combo-start"),
				next: _Next(vec![(
					skill.data.slot_key,
					SkillComboTree {
						skill: combo_skill.clone(),
						next: _Next(vec![]),
					},
				)]),
			},
		)]));
		let (mut app, agent) = setup(skill, combo_tree, slots_item_types);

		app.world
			.entity_mut(agent)
			.insert(ComboTreeRunning(HashMap::from([(
				skill.data.slot_key,
				SkillComboTree {
					skill: skill_usable_with(&[]).named("combo-start"),
					next: _Next(vec![]),
				},
			)])));
		app.update();

		let agent = app.world.entity(agent);
		let slot = agent
			.get::<Slots>()
			.unwrap()
			.0
			.get(&skill.data.slot_key)
			.unwrap();

		assert_eq!(Some(combo_skill.clone()), slot.combo_skill);
	}

	#[test]
	fn call_to_branches_with_combo_start_skill() {
		let skill = &skill_usable_with(&[ItemType::Sword])
			.named("combo-start")
			.active_on(SlotKey::Hand(Side::Off));
		let mut next = Mock_Next::new();
		next.expect_to_vec()
			.times(1)
			.with(eq(skill.clone()))
			.return_const(vec![]);
		next.expect_clone().returning(Mock_Next::new);
		let slots_item_types = &[(skill.data.slot_key, skill.is_usable_with.clone())];
		let combo_tree = ComboTreeTemplate(HashMap::from([(
			skill.data.slot_key,
			SkillComboTree {
				skill: skill_usable_with(&[]).named("combo-start"),
				next,
			},
		)]));

		let (mut app, ..) = setup(skill, combo_tree, slots_item_types);
		app.update();
	}

	#[test]
	fn call_to_branches_with_combo_start_skill_on_running() {
		let skill = &skill_usable_with(&[ItemType::Sword])
			.named("combo-start")
			.active_on(SlotKey::Hand(Side::Off));
		let slots_item_types = &[(skill.data.slot_key, skill.is_usable_with.clone())];
		let combo_tree = ComboTreeTemplate::<Mock_Next>(HashMap::from([]));

		let (mut app, agent) = setup(skill, combo_tree, slots_item_types);
		let mut next = Mock_Next::new();
		next.expect_to_vec()
			.times(1)
			.with(eq(skill.clone()))
			.return_const(vec![]);
		next.expect_clone().returning(Mock_Next::new);
		app.world
			.entity_mut(agent)
			.insert(ComboTreeRunning(HashMap::from([(
				skill.data.slot_key,
				SkillComboTree {
					skill: skill_usable_with(&[]).named("combo-start"),
					next,
				},
			)])));
		app.update();
	}
}
