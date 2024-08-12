pub mod node_entry;
pub mod node_entry_mut;

use super::slots::Slots;
use crate::{
	items::slot_key::SlotKey,
	skills::Skill,
	traits::{Combo, GetCombosOrdered, GetNode, GetNodeMut, PeekNext, TryMap},
};
use bevy::ecs::component::Component;
use common::{
	tools::ordered_hash_map::{Entry, OrderedHashMap},
	traits::{
		get::{Get, GetMut},
		insert::TryInsert,
		iterate::Iterate,
	},
};

#[derive(Component, Clone, PartialEq, Debug)]
pub struct ComboNode<TSkill = Skill>(OrderedHashMap<SlotKey, (TSkill, ComboNode<TSkill>)>);

impl<TSkill> ComboNode<TSkill> {
	pub fn new<const N: usize>(combos: [(SlotKey, (TSkill, ComboNode<TSkill>)); N]) -> Self {
		Self(OrderedHashMap::from(combos))
	}
}

impl<TSkill> Default for ComboNode<TSkill> {
	fn default() -> Self {
		Self(OrderedHashMap::from([]))
	}
}

impl<TKey: Iterate<SlotKey>> Get<TKey, Skill> for ComboNode {
	fn get(&self, slot_key_path: &TKey) -> Option<&Skill> {
		let mut value = None;
		let mut combo_map = &self.0;

		for key in slot_key_path.iterate() {
			let (skill, node) = combo_map.get(key)?;
			value = Some(skill);
			combo_map = &node.0;
		}

		value
	}
}

impl<TKey: Iterate<SlotKey>> GetMut<TKey, Skill> for ComboNode {
	fn get_mut(&mut self, slot_key_path: &TKey) -> Option<&mut Skill> {
		let mut value = None;
		let mut combo_map = &mut self.0;

		for key in slot_key_path.iterate() {
			let (skill, node) = combo_map.get_mut(key)?;
			value = Some(skill);
			combo_map = &mut node.0;
		}

		value
	}
}

#[derive(Debug, PartialEq)]
pub struct NodeEntryMut<'a, TSkill> {
	key: SlotKey,
	tree: &'a mut OrderedHashMap<SlotKey, (TSkill, ComboNode<TSkill>)>,
}

#[derive(Debug, PartialEq)]
pub struct NodeEntry<'a, TSkill> {
	key: SlotKey,
	tree: &'a OrderedHashMap<SlotKey, (TSkill, ComboNode<TSkill>)>,
}

impl<'a, TKey, TSkill> GetNodeMut<'a, TKey> for ComboNode<TSkill>
where
	TKey: Iterate<SlotKey>,
	TSkill: 'a,
{
	type TNode = NodeEntryMut<'a, TSkill>;

	fn node_mut(&'a mut self, slot_key_path: &TKey) -> Option<Self::TNode> {
		let mut slot_key_path = slot_key_path.iterate();
		let mut key = *slot_key_path.next()?;
		let mut tree = &mut self.0;

		for next_key in slot_key_path {
			let (_, node) = tree.get_mut(&key)?;
			key = *next_key;
			tree = &mut node.0;
		}

		Some(NodeEntryMut { key, tree })
	}
}

impl<'a, TKey, TSkill> GetNode<'a, TKey> for ComboNode<TSkill>
where
	TKey: Iterate<SlotKey>,
	TSkill: 'a,
{
	type TNode = NodeEntry<'a, TSkill>;

	fn node(&'a self, slot_key_path: &TKey) -> Option<Self::TNode> {
		let mut slot_key_path = slot_key_path.iterate();
		let mut key = *slot_key_path.next()?;
		let mut tree = &self.0;

		for next_key in slot_key_path {
			let (_, node) = tree.get(&key)?;
			key = *next_key;
			tree = &node.0;
		}

		Some(NodeEntry { key, tree })
	}
}

#[derive(Debug, PartialEq)]
pub enum SlotKeyPathError {
	IsEmpty,
	IsInvalid,
}

impl<TKey: Iterate<SlotKey>> TryInsert<TKey, Skill> for ComboNode {
	type Error = SlotKeyPathError;

	fn try_insert(&mut self, slot_key_path: TKey, value: Skill) -> Result<(), Self::Error> {
		let mut combo_map = &mut self.0;
		let mut slot_key_path = slot_key_path.iterate();
		let mut key = slot_key_path.next().ok_or(SlotKeyPathError::IsEmpty)?;

		for slot_key in slot_key_path {
			let (_, node) = combo_map.get_mut(key).ok_or(SlotKeyPathError::IsInvalid)?;
			combo_map = &mut node.0;
			key = slot_key;
		}

		match combo_map.entry(*key) {
			Entry::Occupied(mut entry) => {
				entry.get_mut().0 = value;
			}
			Entry::Vacant(entry) => {
				entry.insert((value, ComboNode::default()));
			}
		};

		Ok(())
	}
}

impl PeekNext<(Skill, ComboNode)> for ComboNode {
	fn peek_next(&self, trigger: &SlotKey, slots: &Slots) -> Option<(Skill, ComboNode)> {
		let tree = &self.0;
		tree.get(trigger)
			.filter(|(skill, ..)| skill_is_usable(slots, trigger, skill))
			.cloned()
	}
}

fn skill_is_usable(slots: &Slots, trigger: &SlotKey, skill: &Skill) -> bool {
	let Some(slot) = slots.0.get(trigger) else {
		return false;
	};
	let Some(item) = slot.item.as_ref() else {
		return false;
	};
	skill
		.is_usable_with
		.intersection(&item.item_type)
		.next()
		.is_some()
}

fn try_map<TIn, TOut>(
	node: &ComboNode<TIn>,
	map_fn: &mut impl FnMut(&TIn) -> Option<TOut>,
) -> ComboNode<TOut> {
	let combos = node
		.0
		.iter()
		.filter_map(|(key, (skill, next))| Some((*key, (map_fn(skill)?, try_map(next, map_fn)))));

	ComboNode(OrderedHashMap::from_iter(combos))
}

impl<TIn, TOut, TResult: From<ComboNode<TOut>>> TryMap<TIn, TOut, TResult> for ComboNode<TIn> {
	fn try_map(&self, mut map_fn: impl FnMut(&TIn) -> Option<TOut>) -> TResult {
		TResult::from(try_map(self, &mut map_fn))
	}
}

impl GetCombosOrdered for ComboNode {
	fn combos_ordered(&self) -> impl Iterator<Item = Combo> {
		combos(self, vec![])
	}
}

fn combos(combo_node: &ComboNode, key_path: Vec<SlotKey>) -> impl Iterator<Item = Combo> {
	combo_node
		.0
		.iter()
		.map(build_path(key_path))
		.flat_map(append_followup_combo_steps)
}

fn build_path<'a>(
	key_path: Vec<SlotKey>,
) -> impl FnMut((&SlotKey, &'a (Skill, ComboNode))) -> (Vec<SlotKey>, &'a Skill, &'a ComboNode) {
	move |(slot_key, (skill, child_node))| {
		let key_path = [key_path.clone(), vec![*slot_key]].concat();
		(key_path, skill, child_node)
	}
}

fn append_followup_combo_steps<'a>(
	(key_path, skill, child_node): (Vec<SlotKey>, &'a Skill, &'a ComboNode),
) -> Vec<Combo<'a>> {
	let combo_step_key_path = key_path.clone();
	let followup_combo_steps = combos(child_node, combo_step_key_path).collect();
	append_followups((key_path, skill), followup_combo_steps)
}

fn append_followups<'a>(
	combo_step: (Vec<SlotKey>, &'a Skill),
	followups: Vec<Combo<'a>>,
) -> Vec<Combo<'a>> {
	let combo_steps = vec![combo_step];

	if followups.is_empty() {
		return vec![combo_steps];
	}

	followups
		.into_iter()
		.map(|followup_steps| combo_steps.iter().cloned().chain(followup_steps).collect())
		.collect()
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{Item, Mounts, Slot},
		items::ItemType,
	};
	use bevy::{ecs::entity::Entity, prelude::default};
	use common::components::Side;
	use std::collections::{HashMap, HashSet};

	fn slots_main_pistol_off_sword() -> Slots {
		Slots(HashMap::from([
			(
				SlotKey::Hand(Side::Main),
				Slot {
					mounts: Mounts {
						hand: Entity::from_raw(123),
						forearm: Entity::from_raw(456),
					},
					item: Some(Item {
						item_type: HashSet::from([ItemType::Pistol]),
						..default()
					}),
				},
			),
			(
				SlotKey::Hand(Side::Off),
				Slot {
					mounts: Mounts {
						hand: Entity::from_raw(123),
						forearm: Entity::from_raw(456),
					},
					item: Some(Item {
						item_type: HashSet::from([ItemType::Bracer]),
						..default()
					}),
				},
			),
		]))
	}

	#[test]
	fn peek_next_from_tree() {
		let slots = slots_main_pistol_off_sword();
		let node = ComboNode(OrderedHashMap::from([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "first".to_owned(),
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
				ComboNode(OrderedHashMap::from([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "second".to_owned(),
							..default()
						},
						ComboNode(default()),
					),
				)])),
			),
		)]));

		let next: Option<(Skill, ComboNode)> = node.peek_next(&SlotKey::Hand(Side::Main), &slots);

		assert_eq!(
			Some((
				Skill {
					name: "first".to_owned(),
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
				ComboNode(OrderedHashMap::from([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "second".to_owned(),
							..default()
						},
						ComboNode(default()),
					),
				)]))
			)),
			next
		)
	}

	#[test]
	fn peek_none_from_tree_when_slot_on_slot_mismatch() {
		let slots = slots_main_pistol_off_sword();
		let node = ComboNode(OrderedHashMap::from([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "first".to_owned(),
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
				ComboNode(OrderedHashMap::from([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "second".to_owned(),
							..default()
						},
						ComboNode(default()),
					),
				)])),
			),
		)]));

		let next: Option<(Skill, ComboNode)> = node.peek_next(&SlotKey::Hand(Side::Off), &slots);

		assert_eq!(None, next)
	}

	#[test]
	fn peek_none_from_tree_when_slot_on_item_type_mismatch() {
		let slots = slots_main_pistol_off_sword();
		let node = ComboNode(OrderedHashMap::from([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "first".to_owned(),
					is_usable_with: HashSet::from([ItemType::Bracer]),
					..default()
				},
				ComboNode(OrderedHashMap::from([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "second".to_owned(),
							..default()
						},
						ComboNode(default()),
					),
				)])),
			),
		)]));

		let next: Option<(Skill, ComboNode)> = node.peek_next(&SlotKey::Hand(Side::Main), &slots);

		assert_eq!(None, next)
	}

	#[test]
	fn peek_none_from_tree_when_slot_item_none() {
		let mut slots = slots_main_pistol_off_sword();
		slots.0.get_mut(&SlotKey::Hand(Side::Main)).unwrap().item = None;

		let node = ComboNode(OrderedHashMap::from([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "first".to_owned(),
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
				ComboNode(OrderedHashMap::from([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "second".to_owned(),
							..default()
						},
						ComboNode(default()),
					),
				)])),
			),
		)]));

		let next: Option<(Skill, ComboNode)> = node.peek_next(&SlotKey::Hand(Side::Main), &slots);

		assert_eq!(None, next)
	}

	#[test]
	fn peek_none_from_tree_when_slot_none() {
		let mut slots = slots_main_pistol_off_sword();
		slots.0.remove(&SlotKey::Hand(Side::Main));

		let node = ComboNode(OrderedHashMap::from([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "first".to_owned(),
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
				ComboNode(OrderedHashMap::from([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "second".to_owned(),
							..default()
						},
						ComboNode(default()),
					),
				)])),
			),
		)]));

		let next: Option<(Skill, ComboNode)> = node.peek_next(&SlotKey::Hand(Side::Main), &slots);

		assert_eq!(None, next)
	}

	#[test]
	fn get_top_skill() {
		let combos = ComboNode::new([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "some skill".to_owned(),
					..default()
				},
				ComboNode::default(),
			),
		)]);

		let skill = combos.get(&[SlotKey::Hand(Side::Main)]);

		assert_eq!(
			Some(&Skill {
				name: "some skill".to_owned(),
				..default()
			}),
			skill
		);
	}

	#[test]
	fn get_child_skill() {
		let combos = ComboNode::new([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "some skill".to_owned(),
					..default()
				},
				ComboNode::new([(
					SlotKey::Hand(Side::Off),
					(
						Skill {
							name: "some child skill".to_owned(),
							..default()
						},
						ComboNode::default(),
					),
				)]),
			),
		)]);

		let skill = combos.get(&[SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)]);

		assert_eq!(
			Some(&Skill {
				name: "some child skill".to_owned(),
				..default()
			}),
			skill
		);
	}

	#[test]
	fn get_mut_top_skill() {
		let mut combos = ComboNode::new([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "some skill".to_owned(),
					..default()
				},
				ComboNode::default(),
			),
		)]);

		let skill = combos.get_mut(&[SlotKey::Hand(Side::Main)]).unwrap();
		*skill = Skill {
			name: "new skill".to_owned(),
			..default()
		};

		assert_eq!(
			ComboNode::new([(
				SlotKey::Hand(Side::Main),
				(
					Skill {
						name: "new skill".to_owned(),
						..default()
					},
					ComboNode::default(),
				),
			)]),
			combos
		);
	}

	#[test]
	fn get_mut_child_skill() {
		let mut combos = ComboNode::new([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "some skill".to_owned(),
					..default()
				},
				ComboNode::new([(
					SlotKey::Hand(Side::Off),
					(
						Skill {
							name: "some child skill".to_owned(),
							..default()
						},
						ComboNode::default(),
					),
				)]),
			),
		)]);

		let skill = combos
			.get_mut(&[SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)])
			.unwrap();
		*skill = Skill {
			name: "new skill".to_owned(),
			..default()
		};

		assert_eq!(
			ComboNode::new([(
				SlotKey::Hand(Side::Main),
				(
					Skill {
						name: "some skill".to_owned(),
						..default()
					},
					ComboNode::new([(
						SlotKey::Hand(Side::Off),
						(
							Skill {
								name: "new skill".to_owned(),
								..default()
							},
							ComboNode::default(),
						),
					)]),
				),
			)]),
			combos
		);
	}

	#[test]
	fn try_insert_top_skill() {
		let mut combos = ComboNode::default();

		let success = combos.try_insert(
			[SlotKey::Hand(Side::Main)],
			Skill {
				name: "new skill".to_owned(),
				..default()
			},
		);

		assert_eq!(
			(
				ComboNode::new([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "new skill".to_owned(),
							..default()
						},
						ComboNode::default(),
					),
				)]),
				Ok(())
			),
			(combos, success)
		);
	}

	#[test]
	fn try_insert_existing_skill_without_touching_child_skills() {
		let mut combos = ComboNode::new([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "some skill".to_owned(),
					..default()
				},
				ComboNode::new([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "child skill".to_owned(),
							..default()
						},
						ComboNode::default(),
					),
				)]),
			),
		)]);

		let success = combos.try_insert(
			[SlotKey::Hand(Side::Main)],
			Skill {
				name: "new skill".to_owned(),
				..default()
			},
		);

		assert_eq!(
			(
				ComboNode::new([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "new skill".to_owned(),
							..default()
						},
						ComboNode::new([(
							SlotKey::Hand(Side::Main),
							(
								Skill {
									name: "child skill".to_owned(),
									..default()
								},
								ComboNode::default(),
							),
						)]),
					),
				)]),
				Ok(())
			),
			(combos, success)
		);
	}

	#[test]
	fn try_insert_child_skill() {
		let mut combos = ComboNode::new([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "some skill".to_owned(),
					..default()
				},
				ComboNode::default(),
			),
		)]);

		let success = combos.try_insert(
			[SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)],
			Skill {
				name: "new skill".to_owned(),
				..default()
			},
		);

		assert_eq!(
			(
				ComboNode::new([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "some skill".to_owned(),
							..default()
						},
						ComboNode::new([(
							SlotKey::Hand(Side::Off),
							(
								Skill {
									name: "new skill".to_owned(),
									..default()
								},
								ComboNode::default(),
							),
						)]),
					),
				)]),
				Ok(())
			),
			(combos, success)
		);
	}

	#[test]
	fn error_when_provided_keys_empty() {
		let mut combos = ComboNode::default();
		let success = combos.try_insert(
			[],
			Skill {
				name: "new skill".to_owned(),
				..default()
			},
		);

		assert_eq!(
			(ComboNode::default(), Err(SlotKeyPathError::IsEmpty)),
			(combos, success)
		);
	}

	#[test]
	fn error_when_provided_keys_can_not_be_followed() {
		let mut combos = ComboNode::default();
		let success = combos.try_insert(
			[SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Main)],
			Skill {
				name: "new skill".to_owned(),
				..default()
			},
		);

		assert_eq!(
			(ComboNode::default(), Err(SlotKeyPathError::IsInvalid)),
			(combos, success)
		);
	}

	struct _In(&'static str);

	#[derive(Debug, PartialEq)]
	struct _Out(&'static str);

	#[derive(Debug, PartialEq)]
	struct _Result(ComboNode<_Out>);

	impl From<ComboNode<_Out>> for _Result {
		fn from(value: ComboNode<_Out>) -> Self {
			_Result(value)
		}
	}

	#[test]
	fn try_map_skills() {
		let node = ComboNode::new([(
			SlotKey::Hand(Side::Off),
			(_In("my skill"), ComboNode::new([])),
		)]);

		let combos = node.try_map(&mut |value: &_In| Some(_Out(value.0)));

		assert_eq!(
			_Result(ComboNode::new([(
				SlotKey::Hand(Side::Off),
				(_Out("my skill"), ComboNode::new([])),
			)])),
			combos
		);
	}

	#[test]
	fn try_map_child_nodes() {
		let node = ComboNode::new([(
			SlotKey::Hand(Side::Off),
			(
				_In("my skill"),
				ComboNode::new([(
					SlotKey::Hand(Side::Main),
					(_In("my child skill"), ComboNode::new([])),
				)]),
			),
		)]);

		let combos = node.try_map(&mut |value: &_In| Some(_Out(value.0)));

		assert_eq!(
			_Result(ComboNode::new([(
				SlotKey::Hand(Side::Off),
				(
					_Out("my skill"),
					ComboNode::new([(
						SlotKey::Hand(Side::Main),
						(_Out("my child skill"), ComboNode::new([])),
					)])
				),
			)])),
			combos
		);
	}

	#[test]
	fn get_a_mutable_top_entry() {
		let conf = [(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "my skill".to_owned(),
					..default()
				},
				default(),
			),
		)];
		let mut root = ComboNode::new(conf.clone());
		let entry = root.node_mut(&[SlotKey::Hand(Side::Main)]);

		assert_eq!(
			Some(NodeEntryMut {
				key: SlotKey::Hand(Side::Main),
				tree: &mut OrderedHashMap::from(conf),
			}),
			entry,
		)
	}

	#[test]
	fn get_a_mutable_child_entry() {
		let child_conf = [(
			SlotKey::Hand(Side::Off),
			(
				Skill {
					name: "my child skill".to_owned(),
					..default()
				},
				default(),
			),
		)];
		let conf = [(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "my skill".to_owned(),
					..default()
				},
				ComboNode::new(child_conf.clone()),
			),
		)];
		let mut root = ComboNode::new(conf);
		let entry = root.node_mut(&[SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)]);

		assert_eq!(
			Some(NodeEntryMut {
				key: SlotKey::Hand(Side::Off),
				tree: &mut OrderedHashMap::from(child_conf),
			}),
			entry,
		)
	}

	#[test]
	fn get_mutable_none_when_nothing_found_with_key_path() {
		let conf = [(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "my skill".to_owned(),
					..default()
				},
				ComboNode::new([(
					SlotKey::Hand(Side::Off),
					(
						Skill {
							name: "my child skill".to_owned(),
							..default()
						},
						default(),
					),
				)]),
			),
		)];
		let mut root = ComboNode::new(conf);
		let entry = root.node_mut(&[
			SlotKey::Hand(Side::Main),
			SlotKey::Hand(Side::Off),
			SlotKey::Hand(Side::Main),
			SlotKey::Hand(Side::Off),
		]);

		assert_eq!(None, entry)
	}

	#[test]
	fn get_a_mutable_entry_when_only_last_in_key_path_not_found() {
		let conf = [(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "my skill".to_owned(),
					..default()
				},
				ComboNode::new([(
					SlotKey::Hand(Side::Off),
					(
						Skill {
							name: "my child skill".to_owned(),
							..default()
						},
						default(),
					),
				)]),
			),
		)];
		let mut root = ComboNode::new(conf);
		let entry = root.node_mut(&[
			SlotKey::Hand(Side::Main),
			SlotKey::Hand(Side::Off),
			SlotKey::Hand(Side::Main),
		]);

		assert_eq!(
			Some(NodeEntryMut {
				key: SlotKey::Hand(Side::Main),
				tree: &mut OrderedHashMap::default(),
			}),
			entry,
		)
	}

	#[test]
	fn get_a_top_entry() {
		let conf = [(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "my skill".to_owned(),
					..default()
				},
				default(),
			),
		)];
		let root = ComboNode::new(conf.clone());
		let entry = root.node(&[SlotKey::Hand(Side::Main)]);

		assert_eq!(
			Some(NodeEntry {
				key: SlotKey::Hand(Side::Main),
				tree: &OrderedHashMap::from(conf),
			}),
			entry,
		)
	}

	#[test]
	fn get_a_child_entry() {
		let child_conf = [(
			SlotKey::Hand(Side::Off),
			(
				Skill {
					name: "my child skill".to_owned(),
					..default()
				},
				default(),
			),
		)];
		let conf = [(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "my skill".to_owned(),
					..default()
				},
				ComboNode::new(child_conf.clone()),
			),
		)];
		let root = ComboNode::new(conf);
		let entry = root.node(&[SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)]);

		assert_eq!(
			Some(NodeEntry {
				key: SlotKey::Hand(Side::Off),
				tree: &OrderedHashMap::from(child_conf),
			}),
			entry,
		)
	}

	#[test]
	fn get_none_when_nothing_found_with_key_path() {
		let conf = [(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "my skill".to_owned(),
					..default()
				},
				ComboNode::new([(
					SlotKey::Hand(Side::Off),
					(
						Skill {
							name: "my child skill".to_owned(),
							..default()
						},
						default(),
					),
				)]),
			),
		)];
		let root = ComboNode::new(conf);
		let entry = root.node(&[
			SlotKey::Hand(Side::Main),
			SlotKey::Hand(Side::Off),
			SlotKey::Hand(Side::Main),
			SlotKey::Hand(Side::Off),
		]);

		assert_eq!(None, entry)
	}

	#[test]
	fn get_a_usable_entry_when_only_last_in_key_path_not_found() {
		let conf = [(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "my skill".to_owned(),
					..default()
				},
				ComboNode::new([(
					SlotKey::Hand(Side::Off),
					(
						Skill {
							name: "my child skill".to_owned(),
							..default()
						},
						default(),
					),
				)]),
			),
		)];
		let root = ComboNode::new(conf);
		let entry = root.node(&[
			SlotKey::Hand(Side::Main),
			SlotKey::Hand(Side::Off),
			SlotKey::Hand(Side::Main),
		]);

		assert_eq!(
			Some(NodeEntry {
				key: SlotKey::Hand(Side::Main),
				tree: &OrderedHashMap::default(),
			}),
			entry,
		)
	}

	#[test]
	fn get_single_single_combo_with_single_skill() {
		let combos = ComboNode::new([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "some skill".to_owned(),
					..default()
				},
				ComboNode::default(),
			),
		)]);

		assert_eq!(
			vec![vec![(
				vec![SlotKey::Hand(Side::Main)],
				&Skill {
					name: "some skill".to_owned(),
					..default()
				}
			)]],
			combos.combos_ordered().collect::<Vec<_>>()
		)
	}

	#[test]
	fn get_multiple_combos_with_single_skill() {
		let combos = ComboNode::new([
			(
				SlotKey::Hand(Side::Main),
				(
					Skill {
						name: "some right skill".to_owned(),
						..default()
					},
					ComboNode::default(),
				),
			),
			(
				SlotKey::Hand(Side::Off),
				(
					Skill {
						name: "some left skill".to_owned(),
						..default()
					},
					ComboNode::default(),
				),
			),
		]);

		assert_eq!(
			vec![
				vec![(
					vec![SlotKey::Hand(Side::Main)],
					&Skill {
						name: "some right skill".to_owned(),
						..default()
					}
				)],
				vec![(
					vec![SlotKey::Hand(Side::Off)],
					&Skill {
						name: "some left skill".to_owned(),
						..default()
					}
				)]
			],
			combos.combos_ordered().collect::<Vec<_>>()
		)
	}

	#[test]
	fn get_single_combo_with_multiple_skills() {
		let combos = ComboNode::new([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "some skill".to_owned(),
					..default()
				},
				ComboNode::new([(
					SlotKey::Hand(Side::Off),
					(
						Skill {
							name: "some child skill".to_owned(),
							..default()
						},
						ComboNode::default(),
					),
				)]),
			),
		)]);

		assert_eq!(
			vec![vec![
				(
					vec![SlotKey::Hand(Side::Main)],
					&Skill {
						name: "some skill".to_owned(),
						..default()
					}
				),
				(
					vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)],
					&Skill {
						name: "some child skill".to_owned(),
						..default()
					}
				)
			]],
			combos.combos_ordered().collect::<Vec<_>>()
		)
	}

	#[test]
	fn get_multiple_combos_with_multiple_child_skills() {
		let combos = ComboNode::new([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "some skill".to_owned(),
					..default()
				},
				ComboNode::new([
					(
						SlotKey::Hand(Side::Main),
						(
							Skill {
								name: "some right child skill".to_owned(),
								..default()
							},
							ComboNode::default(),
						),
					),
					(
						SlotKey::Hand(Side::Off),
						(
							Skill {
								name: "some left child skill".to_owned(),
								..default()
							},
							ComboNode::default(),
						),
					),
				]),
			),
		)]);

		assert_eq!(
			vec![
				vec![
					(
						vec![SlotKey::Hand(Side::Main)],
						&Skill {
							name: "some skill".to_owned(),
							..default()
						}
					),
					(
						vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Main)],
						&Skill {
							name: "some right child skill".to_owned(),
							..default()
						}
					)
				],
				vec![
					(
						vec![SlotKey::Hand(Side::Main)],
						&Skill {
							name: "some skill".to_owned(),
							..default()
						}
					),
					(
						vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)],
						&Skill {
							name: "some left child skill".to_owned(),
							..default()
						}
					)
				]
			],
			combos.combos_ordered().collect::<Vec<_>>()
		)
	}

	#[test]
	fn get_multiple_combo_with_multiple_deep_child_skills() {
		let combos = ComboNode::new([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "some skill".to_owned(),
					..default()
				},
				ComboNode::new([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "some child skill".to_owned(),
							..default()
						},
						ComboNode::new([
							(
								SlotKey::Hand(Side::Main),
								(
									Skill {
										name: "some right child skill".to_owned(),
										..default()
									},
									ComboNode::default(),
								),
							),
							(
								SlotKey::Hand(Side::Off),
								(
									Skill {
										name: "some left child skill".to_owned(),
										..default()
									},
									ComboNode::default(),
								),
							),
						]),
					),
				)]),
			),
		)]);

		assert_eq!(
			vec![
				vec![
					(
						vec![SlotKey::Hand(Side::Main)],
						&Skill {
							name: "some skill".to_owned(),
							..default()
						}
					),
					(
						vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Main)],
						&Skill {
							name: "some child skill".to_owned(),
							..default()
						}
					),
					(
						vec![
							SlotKey::Hand(Side::Main),
							SlotKey::Hand(Side::Main),
							SlotKey::Hand(Side::Main),
						],
						&Skill {
							name: "some right child skill".to_owned(),
							..default()
						}
					)
				],
				vec![
					(
						vec![SlotKey::Hand(Side::Main)],
						&Skill {
							name: "some skill".to_owned(),
							..default()
						}
					),
					(
						vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Main)],
						&Skill {
							name: "some child skill".to_owned(),
							..default()
						}
					),
					(
						vec![
							SlotKey::Hand(Side::Main),
							SlotKey::Hand(Side::Main),
							SlotKey::Hand(Side::Off),
						],
						&Skill {
							name: "some left child skill".to_owned(),
							..default()
						}
					)
				]
			],
			combos.combos_ordered().collect::<Vec<_>>()
		)
	}

	#[test]
	fn get_multiple_combo_with_multiple_deep_child_skills_with_insertion_order_maintained() {
		let combos = ComboNode::new([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "some skill".to_owned(),
					..default()
				},
				ComboNode::new([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "some child skill".to_owned(),
							..default()
						},
						ComboNode::new([
							(
								SlotKey::Hand(Side::Off),
								(
									Skill {
										name: "some left child skill".to_owned(),
										..default()
									},
									ComboNode::default(),
								),
							),
							(
								SlotKey::Hand(Side::Main),
								(
									Skill {
										name: "some right child skill".to_owned(),
										..default()
									},
									ComboNode::default(),
								),
							),
						]),
					),
				)]),
			),
		)]);

		assert_eq!(
			vec![
				vec![
					(
						vec![SlotKey::Hand(Side::Main)],
						&Skill {
							name: "some skill".to_owned(),
							..default()
						}
					),
					(
						vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Main)],
						&Skill {
							name: "some child skill".to_owned(),
							..default()
						}
					),
					(
						vec![
							SlotKey::Hand(Side::Main),
							SlotKey::Hand(Side::Main),
							SlotKey::Hand(Side::Off),
						],
						&Skill {
							name: "some left child skill".to_owned(),
							..default()
						}
					)
				],
				vec![
					(
						vec![SlotKey::Hand(Side::Main)],
						&Skill {
							name: "some skill".to_owned(),
							..default()
						}
					),
					(
						vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Main)],
						&Skill {
							name: "some child skill".to_owned(),
							..default()
						}
					),
					(
						vec![
							SlotKey::Hand(Side::Main),
							SlotKey::Hand(Side::Main),
							SlotKey::Hand(Side::Main),
						],
						&Skill {
							name: "some right child skill".to_owned(),
							..default()
						}
					)
				]
			],
			combos.combos_ordered().collect::<Vec<_>>()
		)
	}
}
