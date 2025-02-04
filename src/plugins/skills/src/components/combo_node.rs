pub mod node_entry;
pub mod node_entry_mut;

use std::collections::VecDeque;

use crate::{
	skills::Skill,
	traits::{Combo, GetCombosOrdered, GetNode, GetNodeMut, PeekNext, RootKeys},
};
use bevy::ecs::component::Component;
use common::{
	tools::{
		item_type::ItemType,
		ordered_hash_map::{Entry, OrderedHashMap},
		slot_key::SlotKey,
	},
	traits::{
		accessors::get::{GetMut, GetRef},
		handles_equipment::GetFollowupKeys,
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

impl<TKey: Iterate<SlotKey>> GetRef<TKey, Skill> for ComboNode {
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

impl<TKey, TSkill> GetNodeMut<TKey> for ComboNode<TSkill>
where
	TKey: Iterate<SlotKey>,
{
	type TNode<'a>
		= NodeEntryMut<'a, TSkill>
	where
		Self: 'a;

	fn node_mut<'a>(&'a mut self, slot_key_path: &TKey) -> Option<Self::TNode<'a>> {
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

impl<TKey, TSkill> GetNode<TKey> for ComboNode<TSkill>
where
	TKey: Iterate<SlotKey>,
{
	type TNode<'a>
		= NodeEntry<'a, TSkill>
	where
		Self: 'a;

	fn node<'a>(&'a self, slot_key_path: &TKey) -> Option<Self::TNode<'a>> {
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

impl RootKeys for ComboNode {
	type TItem = SlotKey;

	fn root_keys(&self) -> impl Iterator<Item = Self::TItem> {
		self.0.keys().cloned()
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
	fn peek_next(&self, trigger: &SlotKey, item_type: &ItemType) -> Option<(Skill, ComboNode)> {
		let ComboNode(tree) = self;
		let (skill, combo) = tree.get(trigger)?;

		if !skill.is_usable_with.contains(item_type) {
			return None;
		}

		Some((skill.clone(), combo.clone()))
	}
}

impl GetCombosOrdered for ComboNode {
	fn combos_ordered(&self) -> impl Iterator<Item = Combo> {
		combos(self, vec![])
	}
}

impl GetFollowupKeys for ComboNode {
	type TKey = SlotKey;

	fn followup_keys<T>(&self, after: T) -> Option<Vec<Self::TKey>>
	where
		T: Into<VecDeque<Self::TKey>>,
	{
		if self.0.is_empty() {
			return Some(vec![]);
		}

		let mut after: VecDeque<Self::TKey> = after.into();

		let Some(key) = after.pop_front() else {
			return Some(self.0.keys().copied().collect());
		};

		let (_, next) = self.0.get(&key)?;

		next.followup_keys(after)
	}
}

fn combos(combo_node: &ComboNode, key_path: Vec<SlotKey>) -> impl Iterator<Item = Combo> {
	combo_node
		.0
		.iter()
		.map(build_path(key_path))
		.flat_map(append_followup_combo_steps)
}

#[allow(clippy::type_complexity)]
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
	use bevy::prelude::default;
	use common::tools::slot_key::Side;
	use std::collections::HashSet;

	#[test]
	fn peek_next_from_tree() {
		let node = ComboNode(OrderedHashMap::from([(
			SlotKey::BottomHand(Side::Right),
			(
				Skill {
					name: "first".to_owned(),
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
				ComboNode(OrderedHashMap::from([(
					SlotKey::BottomHand(Side::Right),
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

		let next = node.peek_next(&SlotKey::BottomHand(Side::Right), &ItemType::Pistol);

		assert_eq!(
			Some((
				Skill {
					name: "first".to_owned(),
					is_usable_with: HashSet::from([ItemType::Pistol]),
					..default()
				},
				ComboNode(OrderedHashMap::from([(
					SlotKey::BottomHand(Side::Right),
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
	fn peek_none_if_item_type_not_usable() {
		let node = ComboNode(OrderedHashMap::from([(
			SlotKey::BottomHand(Side::Right),
			(
				Skill {
					name: "first".to_owned(),
					is_usable_with: HashSet::from([ItemType::Bracer]),
					..default()
				},
				ComboNode(OrderedHashMap::from([(
					SlotKey::BottomHand(Side::Right),
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

		let next = node.peek_next(&SlotKey::BottomHand(Side::Right), &ItemType::Pistol);

		assert_eq!(None as Option<(Skill, ComboNode)>, next)
	}

	#[test]
	fn get_top_skill() {
		let combos = ComboNode::new([(
			SlotKey::BottomHand(Side::Right),
			(
				Skill {
					name: "some skill".to_owned(),
					..default()
				},
				ComboNode::default(),
			),
		)]);

		let skill = combos.get(&[SlotKey::BottomHand(Side::Right)]);

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
			SlotKey::BottomHand(Side::Right),
			(
				Skill {
					name: "some skill".to_owned(),
					..default()
				},
				ComboNode::new([(
					SlotKey::BottomHand(Side::Left),
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

		let skill = combos.get(&[
			SlotKey::BottomHand(Side::Right),
			SlotKey::BottomHand(Side::Left),
		]);

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
			SlotKey::BottomHand(Side::Right),
			(
				Skill {
					name: "some skill".to_owned(),
					..default()
				},
				ComboNode::default(),
			),
		)]);

		let skill = combos.get_mut(&[SlotKey::BottomHand(Side::Right)]).unwrap();
		*skill = Skill {
			name: "new skill".to_owned(),
			..default()
		};

		assert_eq!(
			ComboNode::new([(
				SlotKey::BottomHand(Side::Right),
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
			SlotKey::BottomHand(Side::Right),
			(
				Skill {
					name: "some skill".to_owned(),
					..default()
				},
				ComboNode::new([(
					SlotKey::BottomHand(Side::Left),
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
			.get_mut(&[
				SlotKey::BottomHand(Side::Right),
				SlotKey::BottomHand(Side::Left),
			])
			.unwrap();
		*skill = Skill {
			name: "new skill".to_owned(),
			..default()
		};

		assert_eq!(
			ComboNode::new([(
				SlotKey::BottomHand(Side::Right),
				(
					Skill {
						name: "some skill".to_owned(),
						..default()
					},
					ComboNode::new([(
						SlotKey::BottomHand(Side::Left),
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
			[SlotKey::BottomHand(Side::Right)],
			Skill {
				name: "new skill".to_owned(),
				..default()
			},
		);

		assert_eq!(
			(
				ComboNode::new([(
					SlotKey::BottomHand(Side::Right),
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
			SlotKey::BottomHand(Side::Right),
			(
				Skill {
					name: "some skill".to_owned(),
					..default()
				},
				ComboNode::new([(
					SlotKey::BottomHand(Side::Right),
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
			[SlotKey::BottomHand(Side::Right)],
			Skill {
				name: "new skill".to_owned(),
				..default()
			},
		);

		assert_eq!(
			(
				ComboNode::new([(
					SlotKey::BottomHand(Side::Right),
					(
						Skill {
							name: "new skill".to_owned(),
							..default()
						},
						ComboNode::new([(
							SlotKey::BottomHand(Side::Right),
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
			SlotKey::BottomHand(Side::Right),
			(
				Skill {
					name: "some skill".to_owned(),
					..default()
				},
				ComboNode::default(),
			),
		)]);

		let success = combos.try_insert(
			[
				SlotKey::BottomHand(Side::Right),
				SlotKey::BottomHand(Side::Left),
			],
			Skill {
				name: "new skill".to_owned(),
				..default()
			},
		);

		assert_eq!(
			(
				ComboNode::new([(
					SlotKey::BottomHand(Side::Right),
					(
						Skill {
							name: "some skill".to_owned(),
							..default()
						},
						ComboNode::new([(
							SlotKey::BottomHand(Side::Left),
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
			[
				SlotKey::BottomHand(Side::Right),
				SlotKey::BottomHand(Side::Right),
			],
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
	fn get_a_mutable_top_entry() {
		let conf = [(
			SlotKey::BottomHand(Side::Right),
			(
				Skill {
					name: "my skill".to_owned(),
					..default()
				},
				default(),
			),
		)];
		let mut root = ComboNode::new(conf.clone());
		let entry = root.node_mut(&[SlotKey::BottomHand(Side::Right)]);

		assert_eq!(
			Some(NodeEntryMut {
				key: SlotKey::BottomHand(Side::Right),
				tree: &mut OrderedHashMap::from(conf),
			}),
			entry,
		)
	}

	#[test]
	fn get_a_mutable_child_entry() {
		let child_conf = [(
			SlotKey::BottomHand(Side::Left),
			(
				Skill {
					name: "my child skill".to_owned(),
					..default()
				},
				default(),
			),
		)];
		let conf = [(
			SlotKey::BottomHand(Side::Right),
			(
				Skill {
					name: "my skill".to_owned(),
					..default()
				},
				ComboNode::new(child_conf.clone()),
			),
		)];
		let mut root = ComboNode::new(conf);
		let entry = root.node_mut(&[
			SlotKey::BottomHand(Side::Right),
			SlotKey::BottomHand(Side::Left),
		]);

		assert_eq!(
			Some(NodeEntryMut {
				key: SlotKey::BottomHand(Side::Left),
				tree: &mut OrderedHashMap::from(child_conf),
			}),
			entry,
		)
	}

	#[test]
	fn get_mutable_none_when_nothing_found_with_key_path() {
		let conf = [(
			SlotKey::BottomHand(Side::Right),
			(
				Skill {
					name: "my skill".to_owned(),
					..default()
				},
				ComboNode::new([(
					SlotKey::BottomHand(Side::Left),
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
			SlotKey::BottomHand(Side::Right),
			SlotKey::BottomHand(Side::Left),
			SlotKey::BottomHand(Side::Right),
			SlotKey::BottomHand(Side::Left),
		]);

		assert_eq!(None, entry)
	}

	#[test]
	fn get_a_mutable_entry_when_only_last_in_key_path_not_found() {
		let conf = [(
			SlotKey::BottomHand(Side::Right),
			(
				Skill {
					name: "my skill".to_owned(),
					..default()
				},
				ComboNode::new([(
					SlotKey::BottomHand(Side::Left),
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
			SlotKey::BottomHand(Side::Right),
			SlotKey::BottomHand(Side::Left),
			SlotKey::BottomHand(Side::Right),
		]);

		assert_eq!(
			Some(NodeEntryMut {
				key: SlotKey::BottomHand(Side::Right),
				tree: &mut OrderedHashMap::default(),
			}),
			entry,
		)
	}

	#[test]
	fn get_a_top_entry() {
		let conf = [(
			SlotKey::BottomHand(Side::Right),
			(
				Skill {
					name: "my skill".to_owned(),
					..default()
				},
				default(),
			),
		)];
		let root = ComboNode::new(conf.clone());
		let entry = root.node(&[SlotKey::BottomHand(Side::Right)]);

		assert_eq!(
			Some(NodeEntry {
				key: SlotKey::BottomHand(Side::Right),
				tree: &OrderedHashMap::from(conf),
			}),
			entry,
		)
	}

	#[test]
	fn get_a_child_entry() {
		let child_conf = [(
			SlotKey::BottomHand(Side::Left),
			(
				Skill {
					name: "my child skill".to_owned(),
					..default()
				},
				default(),
			),
		)];
		let conf = [(
			SlotKey::BottomHand(Side::Right),
			(
				Skill {
					name: "my skill".to_owned(),
					..default()
				},
				ComboNode::new(child_conf.clone()),
			),
		)];
		let root = ComboNode::new(conf);
		let entry = root.node(&[
			SlotKey::BottomHand(Side::Right),
			SlotKey::BottomHand(Side::Left),
		]);

		assert_eq!(
			Some(NodeEntry {
				key: SlotKey::BottomHand(Side::Left),
				tree: &OrderedHashMap::from(child_conf),
			}),
			entry,
		)
	}

	#[test]
	fn get_none_when_nothing_found_with_key_path() {
		let conf = [(
			SlotKey::BottomHand(Side::Right),
			(
				Skill {
					name: "my skill".to_owned(),
					..default()
				},
				ComboNode::new([(
					SlotKey::BottomHand(Side::Left),
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
			SlotKey::BottomHand(Side::Right),
			SlotKey::BottomHand(Side::Left),
			SlotKey::BottomHand(Side::Right),
			SlotKey::BottomHand(Side::Left),
		]);

		assert_eq!(None, entry)
	}

	#[test]
	fn get_a_usable_entry_when_only_last_in_key_path_not_found() {
		let conf = [(
			SlotKey::BottomHand(Side::Right),
			(
				Skill {
					name: "my skill".to_owned(),
					..default()
				},
				ComboNode::new([(
					SlotKey::BottomHand(Side::Left),
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
			SlotKey::BottomHand(Side::Right),
			SlotKey::BottomHand(Side::Left),
			SlotKey::BottomHand(Side::Right),
		]);

		assert_eq!(
			Some(NodeEntry {
				key: SlotKey::BottomHand(Side::Right),
				tree: &OrderedHashMap::default(),
			}),
			entry,
		)
	}

	#[test]
	fn get_root_keys() {
		let combos = ComboNode::new([(
			SlotKey::BottomHand(Side::Right),
			(Skill::default(), ComboNode::default()),
		)]);

		assert_eq!(
			vec![SlotKey::BottomHand(Side::Right)],
			combos.root_keys().collect::<Vec<_>>()
		);
	}

	#[test]
	fn get_root_keys_empty() {
		let combos = ComboNode::new([]);

		assert_eq!(
			vec![] as Vec<SlotKey>,
			combos.root_keys().collect::<Vec<_>>()
		);
	}

	#[test]
	fn get_single_single_combo_with_single_skill() {
		let combos = ComboNode::new([(
			SlotKey::BottomHand(Side::Right),
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
				vec![SlotKey::BottomHand(Side::Right)],
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
				SlotKey::BottomHand(Side::Right),
				(
					Skill {
						name: "some right skill".to_owned(),
						..default()
					},
					ComboNode::default(),
				),
			),
			(
				SlotKey::BottomHand(Side::Left),
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
					vec![SlotKey::BottomHand(Side::Right)],
					&Skill {
						name: "some right skill".to_owned(),
						..default()
					}
				)],
				vec![(
					vec![SlotKey::BottomHand(Side::Left)],
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
			SlotKey::BottomHand(Side::Right),
			(
				Skill {
					name: "some skill".to_owned(),
					..default()
				},
				ComboNode::new([(
					SlotKey::BottomHand(Side::Left),
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
					vec![SlotKey::BottomHand(Side::Right)],
					&Skill {
						name: "some skill".to_owned(),
						..default()
					}
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Left)
					],
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
			SlotKey::BottomHand(Side::Right),
			(
				Skill {
					name: "some skill".to_owned(),
					..default()
				},
				ComboNode::new([
					(
						SlotKey::BottomHand(Side::Right),
						(
							Skill {
								name: "some right child skill".to_owned(),
								..default()
							},
							ComboNode::default(),
						),
					),
					(
						SlotKey::BottomHand(Side::Left),
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
						vec![SlotKey::BottomHand(Side::Right)],
						&Skill {
							name: "some skill".to_owned(),
							..default()
						}
					),
					(
						vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right)
						],
						&Skill {
							name: "some right child skill".to_owned(),
							..default()
						}
					)
				],
				vec![
					(
						vec![SlotKey::BottomHand(Side::Right)],
						&Skill {
							name: "some skill".to_owned(),
							..default()
						}
					),
					(
						vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Left)
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
	fn get_multiple_combo_with_multiple_deep_child_skills() {
		let combos = ComboNode::new([(
			SlotKey::BottomHand(Side::Right),
			(
				Skill {
					name: "some skill".to_owned(),
					..default()
				},
				ComboNode::new([(
					SlotKey::BottomHand(Side::Right),
					(
						Skill {
							name: "some child skill".to_owned(),
							..default()
						},
						ComboNode::new([
							(
								SlotKey::BottomHand(Side::Right),
								(
									Skill {
										name: "some right child skill".to_owned(),
										..default()
									},
									ComboNode::default(),
								),
							),
							(
								SlotKey::BottomHand(Side::Left),
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
						vec![SlotKey::BottomHand(Side::Right)],
						&Skill {
							name: "some skill".to_owned(),
							..default()
						}
					),
					(
						vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right)
						],
						&Skill {
							name: "some child skill".to_owned(),
							..default()
						}
					),
					(
						vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
						],
						&Skill {
							name: "some right child skill".to_owned(),
							..default()
						}
					)
				],
				vec![
					(
						vec![SlotKey::BottomHand(Side::Right)],
						&Skill {
							name: "some skill".to_owned(),
							..default()
						}
					),
					(
						vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right)
						],
						&Skill {
							name: "some child skill".to_owned(),
							..default()
						}
					),
					(
						vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Left),
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
			SlotKey::BottomHand(Side::Right),
			(
				Skill {
					name: "some skill".to_owned(),
					..default()
				},
				ComboNode::new([(
					SlotKey::BottomHand(Side::Right),
					(
						Skill {
							name: "some child skill".to_owned(),
							..default()
						},
						ComboNode::new([
							(
								SlotKey::BottomHand(Side::Left),
								(
									Skill {
										name: "some left child skill".to_owned(),
										..default()
									},
									ComboNode::default(),
								),
							),
							(
								SlotKey::BottomHand(Side::Right),
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
						vec![SlotKey::BottomHand(Side::Right)],
						&Skill {
							name: "some skill".to_owned(),
							..default()
						}
					),
					(
						vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right)
						],
						&Skill {
							name: "some child skill".to_owned(),
							..default()
						}
					),
					(
						vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Left),
						],
						&Skill {
							name: "some left child skill".to_owned(),
							..default()
						}
					)
				],
				vec![
					(
						vec![SlotKey::BottomHand(Side::Right)],
						&Skill {
							name: "some skill".to_owned(),
							..default()
						}
					),
					(
						vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right)
						],
						&Skill {
							name: "some child skill".to_owned(),
							..default()
						}
					),
					(
						vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
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

	#[test]
	fn get_followup_keys_of_empty_path() {
		let node = ComboNode::new([
			(
				SlotKey::TopHand(Side::Right),
				(Skill::default(), ComboNode::default()),
			),
			(
				SlotKey::TopHand(Side::Left),
				(Skill::default(), ComboNode::default()),
			),
		]);

		let followup_keys = node.followup_keys(vec![]);

		assert_eq!(
			Some(vec![
				SlotKey::TopHand(Side::Right),
				SlotKey::TopHand(Side::Left)
			]),
			followup_keys
		);
	}

	#[test]
	fn get_followup_keys_empty_if_combo_empty() {
		let node = ComboNode::default();

		let followup_keys = node.followup_keys(vec![]);

		assert_eq!(Some(vec![]), followup_keys);
	}

	#[test]
	fn get_followup_keys_of_target_path() {
		let node = ComboNode::new([(
			SlotKey::TopHand(Side::Left),
			(
				Skill::default(),
				ComboNode::new([(
					SlotKey::BottomHand(Side::Left),
					(
						Skill::default(),
						ComboNode::new([
							(
								SlotKey::BottomHand(Side::Right),
								(Skill::default(), ComboNode::default()),
							),
							(
								SlotKey::BottomHand(Side::Left),
								(Skill::default(), ComboNode::default()),
							),
						]),
					),
				)]),
			),
		)]);

		let followup_keys = node.followup_keys(vec![
			SlotKey::TopHand(Side::Left),
			SlotKey::BottomHand(Side::Left),
		]);

		assert_eq!(
			Some(vec![
				SlotKey::BottomHand(Side::Right),
				SlotKey::BottomHand(Side::Left)
			]),
			followup_keys
		);
	}

	#[test]
	fn get_followup_keys_none_of_invalid_target_path() {
		let node = ComboNode::new([(
			SlotKey::TopHand(Side::Left),
			(
				Skill::default(),
				ComboNode::new([(
					SlotKey::BottomHand(Side::Left),
					(Skill::default(), ComboNode::default()),
				)]),
			),
		)]);

		let followup_keys = node.followup_keys(vec![
			SlotKey::TopHand(Side::Left),
			SlotKey::BottomHand(Side::Right),
		]);

		assert_eq!(None, followup_keys);
	}
}
