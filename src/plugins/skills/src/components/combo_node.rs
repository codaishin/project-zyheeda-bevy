pub(crate) mod dto;
pub(crate) mod node_entry_mut;

use crate::{
	skills::Skill,
	traits::{
		GetNode,
		GetNodeMut,
		follow_up_keys::FollowupKeys,
		peek_next_recursive::PeekNextRecursive,
	},
};
use bevy::ecs::component::Component;
use common::{
	tools::{
		action_key::slot::{Combo, SlotKey},
		item_type::{CompatibleItems, ItemType},
		ordered_hash_map::{Entry, IntoIter, OrderedHashMap},
	},
	traits::{
		accessors::get::{GetMut, GetRef},
		handles_combo_menu::GetCombosOrdered,
		insert::TryInsert,
		iterate::Iterate,
	},
};
use std::collections::VecDeque;

#[derive(Component, Clone, PartialEq, Debug)]
pub struct ComboNode<TSkill = Skill>(OrderedHashMap<SlotKey, (TSkill, ComboNode<TSkill>)>);

#[cfg(test)]
impl<TSkill> ComboNode<TSkill> {
	pub fn new<const N: usize>(combos: [(SlotKey, (TSkill, ComboNode<TSkill>)); N]) -> Self {
		Self(OrderedHashMap::from(combos))
	}
}

impl IntoIterator for ComboNode {
	type Item = (SlotKey, (Skill, ComboNode));
	type IntoIter = IntoIter<SlotKey, (Skill, ComboNode)>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

impl<TSkill> Default for ComboNode<TSkill> {
	fn default() -> Self {
		Self(OrderedHashMap::from([]))
	}
}

impl<TKey> GetRef<TKey, Skill> for ComboNode
where
	for<'a> TKey: Iterate<'a, TItem = &'a SlotKey> + 'a,
{
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

impl<TKey> GetMut<TKey, Skill> for ComboNode
where
	for<'a> TKey: Iterate<'a, TItem = &'a SlotKey> + 'a,
{
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
	for<'a> TKey: Iterate<'a, TItem = &'a SlotKey> + 'a,
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
	for<'a> TKey: Iterate<'a, TItem = &'a SlotKey> + 'a,
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

#[derive(Debug, PartialEq)]
pub enum SlotKeyPathError {
	IsEmpty,
	IsInvalid,
}

impl<TKey> TryInsert<TKey, Skill> for ComboNode
where
	for<'a> TKey: Iterate<'a, TItem = &'a SlotKey> + 'a,
{
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

impl PeekNextRecursive for ComboNode {
	type TNext<'a>
		= &'a Skill
	where
		Self: 'a;

	type TRecursiveNode<'a>
		= &'a Self
	where
		Self: 'a;

	fn peek_next_recursive<'a>(
		&'a self,
		trigger: &SlotKey,
		item_type: &ItemType,
	) -> Option<(Self::TNext<'a>, Self::TRecursiveNode<'a>)> {
		let ComboNode(tree) = self;
		let (skill, combo) = tree.get(trigger)?;
		let CompatibleItems(is_usable_with) = &skill.compatible_items;

		if !is_usable_with.contains(item_type) {
			return None;
		}

		Some((skill, combo))
	}
}

impl GetCombosOrdered<Skill> for ComboNode {
	fn combos_ordered(&self) -> Vec<Combo<Skill>> {
		combos(self, vec![])
	}
}

impl FollowupKeys for ComboNode {
	fn followup_keys<T>(&self, after: T) -> Option<Vec<SlotKey>>
	where
		T: Into<VecDeque<SlotKey>>,
	{
		let mut after: VecDeque<SlotKey> = after.into();

		let Some(key) = after.pop_front() else {
			return Some(self.0.keys().copied().collect());
		};

		let (_, next) = self.0.get(&key)?;

		next.followup_keys(after)
	}
}

fn combos(combo_node: &ComboNode, key_path: Vec<SlotKey>) -> Vec<Combo<Skill>> {
	combo_node
		.0
		.iter()
		.map(build_path(key_path))
		.flat_map(append_followup_combo_steps)
		.collect()
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

fn append_followup_combo_steps(
	(key_path, skill, child_node): (Vec<SlotKey>, &Skill, &ComboNode),
) -> Vec<Combo<Skill>> {
	let followup_combo_steps = combos(child_node, key_path.clone());
	append_followups((key_path, skill), followup_combo_steps)
}

fn append_followups(
	(combo_path, skill): (Vec<SlotKey>, &Skill),
	followups: Vec<Combo<Skill>>,
) -> Vec<Combo<Skill>> {
	let combo_steps = vec![(combo_path, skill.clone())];

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
	use common::{tools::action_key::slot::Side, traits::handles_localization::Token};
	use std::collections::HashSet;

	#[test]
	fn peek_next_from_tree() {
		let node = ComboNode(OrderedHashMap::from([(
			SlotKey::BottomHand(Side::Right),
			(
				Skill {
					token: Token::from("first"),
					compatible_items: CompatibleItems(HashSet::from([ItemType::Pistol])),
					..default()
				},
				ComboNode(OrderedHashMap::from([(
					SlotKey::BottomHand(Side::Right),
					(
						Skill {
							token: Token::from("second"),
							..default()
						},
						ComboNode(default()),
					),
				)])),
			),
		)]));

		let next = node.peek_next_recursive(&SlotKey::BottomHand(Side::Right), &ItemType::Pistol);

		assert_eq!(
			Some((
				&Skill {
					token: Token::from("first"),
					compatible_items: CompatibleItems(HashSet::from([ItemType::Pistol])),
					..default()
				},
				&ComboNode(OrderedHashMap::from([(
					SlotKey::BottomHand(Side::Right),
					(
						Skill {
							token: Token::from("second"),
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
					token: Token::from("first"),
					compatible_items: CompatibleItems(HashSet::from([ItemType::Bracer])),
					..default()
				},
				ComboNode(OrderedHashMap::from([(
					SlotKey::BottomHand(Side::Right),
					(
						Skill {
							token: Token::from("second"),
							..default()
						},
						ComboNode(default()),
					),
				)])),
			),
		)]));

		let next = node.peek_next_recursive(&SlotKey::BottomHand(Side::Right), &ItemType::Pistol);

		assert_eq!(None as Option<(&Skill, &ComboNode)>, next)
	}

	#[test]
	fn get_top_skill() {
		let combos = ComboNode::new([(
			SlotKey::BottomHand(Side::Right),
			(
				Skill {
					token: Token::from("some skill"),
					..default()
				},
				ComboNode::default(),
			),
		)]);

		let skill = combos.get(&[SlotKey::BottomHand(Side::Right)]);

		assert_eq!(
			Some(&Skill {
				token: Token::from("some skill"),
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
					token: Token::from("some skill"),
					..default()
				},
				ComboNode::new([(
					SlotKey::BottomHand(Side::Left),
					(
						Skill {
							token: Token::from("some child skill"),
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
				token: Token::from("some child skill"),
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
					token: Token::from("some skill"),
					..default()
				},
				ComboNode::default(),
			),
		)]);

		let skill = combos.get_mut(&[SlotKey::BottomHand(Side::Right)]).unwrap();
		*skill = Skill {
			token: Token::from("new skill"),
			..default()
		};

		assert_eq!(
			ComboNode::new([(
				SlotKey::BottomHand(Side::Right),
				(
					Skill {
						token: Token::from("new skill"),
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
					token: Token::from("some skill"),
					..default()
				},
				ComboNode::new([(
					SlotKey::BottomHand(Side::Left),
					(
						Skill {
							token: Token::from("some child skill"),
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
			token: Token::from("new skill"),
			..default()
		};

		assert_eq!(
			ComboNode::new([(
				SlotKey::BottomHand(Side::Right),
				(
					Skill {
						token: Token::from("some skill"),
						..default()
					},
					ComboNode::new([(
						SlotKey::BottomHand(Side::Left),
						(
							Skill {
								token: Token::from("new skill"),
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
				token: Token::from("new skill"),
				..default()
			},
		);

		assert_eq!(
			(
				ComboNode::new([(
					SlotKey::BottomHand(Side::Right),
					(
						Skill {
							token: Token::from("new skill"),
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
					token: Token::from("some skill"),
					..default()
				},
				ComboNode::new([(
					SlotKey::BottomHand(Side::Right),
					(
						Skill {
							token: Token::from("child skill"),
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
				token: Token::from("new skill"),
				..default()
			},
		);

		assert_eq!(
			(
				ComboNode::new([(
					SlotKey::BottomHand(Side::Right),
					(
						Skill {
							token: Token::from("new skill"),
							..default()
						},
						ComboNode::new([(
							SlotKey::BottomHand(Side::Right),
							(
								Skill {
									token: Token::from("child skill"),
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
					token: Token::from("some skill"),
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
				token: Token::from("new skill"),
				..default()
			},
		);

		assert_eq!(
			(
				ComboNode::new([(
					SlotKey::BottomHand(Side::Right),
					(
						Skill {
							token: Token::from("some skill"),
							..default()
						},
						ComboNode::new([(
							SlotKey::BottomHand(Side::Left),
							(
								Skill {
									token: Token::from("new skill"),
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
				token: Token::from("new skill"),
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
				token: Token::from("new skill"),
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
					token: Token::from("my skill"),
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
					token: Token::from("my child skill"),
					..default()
				},
				default(),
			),
		)];
		let conf = [(
			SlotKey::BottomHand(Side::Right),
			(
				Skill {
					token: Token::from("my skill"),
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
					token: Token::from("my skill"),
					..default()
				},
				ComboNode::new([(
					SlotKey::BottomHand(Side::Left),
					(
						Skill {
							token: Token::from("my child skill"),
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
					token: Token::from("my skill"),
					..default()
				},
				ComboNode::new([(
					SlotKey::BottomHand(Side::Left),
					(
						Skill {
							token: Token::from("my child skill"),
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
					token: Token::from("my skill"),
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
					token: Token::from("my child skill"),
					..default()
				},
				default(),
			),
		)];
		let conf = [(
			SlotKey::BottomHand(Side::Right),
			(
				Skill {
					token: Token::from("my skill"),
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
					token: Token::from("my skill"),
					..default()
				},
				ComboNode::new([(
					SlotKey::BottomHand(Side::Left),
					(
						Skill {
							token: Token::from("my child skill"),
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
					token: Token::from("my skill"),
					..default()
				},
				ComboNode::new([(
					SlotKey::BottomHand(Side::Left),
					(
						Skill {
							token: Token::from("my child skill"),
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
	fn get_single_single_combo_with_single_skill() {
		let combos = ComboNode::new([(
			SlotKey::BottomHand(Side::Right),
			(
				Skill {
					token: Token::from("some skill"),
					..default()
				},
				ComboNode::default(),
			),
		)]);

		assert_eq!(
			vec![vec![(
				vec![SlotKey::BottomHand(Side::Right)],
				Skill {
					token: Token::from("some skill"),
					..default()
				}
			)]],
			combos.combos_ordered()
		)
	}

	#[test]
	fn get_multiple_combos_with_single_skill() {
		let combos = ComboNode::new([
			(
				SlotKey::BottomHand(Side::Right),
				(
					Skill {
						token: Token::from("some right skill"),
						..default()
					},
					ComboNode::default(),
				),
			),
			(
				SlotKey::BottomHand(Side::Left),
				(
					Skill {
						token: Token::from("some left skill"),
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
					Skill {
						token: Token::from("some right skill"),
						..default()
					},
				)],
				vec![(
					vec![SlotKey::BottomHand(Side::Left)],
					Skill {
						token: Token::from("some left skill"),
						..default()
					},
				)]
			],
			combos.combos_ordered()
		)
	}

	#[test]
	fn get_single_combo_with_multiple_skills() {
		let combos = ComboNode::new([(
			SlotKey::BottomHand(Side::Right),
			(
				Skill {
					token: Token::from("some skill"),
					..default()
				},
				ComboNode::new([(
					SlotKey::BottomHand(Side::Left),
					(
						Skill {
							token: Token::from("some child skill"),
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
					Skill {
						token: Token::from("some skill"),
						..default()
					},
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Left)
					],
					Skill {
						token: Token::from("some child skill"),
						..default()
					},
				)
			]],
			combos.combos_ordered()
		)
	}

	#[test]
	fn get_multiple_combos_with_multiple_child_skills() {
		let combos = ComboNode::new([(
			SlotKey::BottomHand(Side::Right),
			(
				Skill {
					token: Token::from("some skill"),
					..default()
				},
				ComboNode::new([
					(
						SlotKey::BottomHand(Side::Right),
						(
							Skill {
								token: Token::from("some right child skill"),
								..default()
							},
							ComboNode::default(),
						),
					),
					(
						SlotKey::BottomHand(Side::Left),
						(
							Skill {
								token: Token::from("some left child skill"),
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
						Skill {
							token: Token::from("some skill"),
							..default()
						},
					),
					(
						vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right)
						],
						Skill {
							token: Token::from("some right child skill"),
							..default()
						},
					)
				],
				vec![
					(
						vec![SlotKey::BottomHand(Side::Right)],
						Skill {
							token: Token::from("some skill"),
							..default()
						},
					),
					(
						vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Left)
						],
						Skill {
							token: Token::from("some left child skill"),
							..default()
						},
					)
				]
			],
			combos.combos_ordered()
		)
	}

	#[test]
	fn get_multiple_combo_with_multiple_deep_child_skills() {
		let combos = ComboNode::new([(
			SlotKey::BottomHand(Side::Right),
			(
				Skill {
					token: Token::from("some skill"),
					..default()
				},
				ComboNode::new([(
					SlotKey::BottomHand(Side::Right),
					(
						Skill {
							token: Token::from("some child skill"),
							..default()
						},
						ComboNode::new([
							(
								SlotKey::BottomHand(Side::Right),
								(
									Skill {
										token: Token::from("some right child skill"),
										..default()
									},
									ComboNode::default(),
								),
							),
							(
								SlotKey::BottomHand(Side::Left),
								(
									Skill {
										token: Token::from("some left child skill"),
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
						Skill {
							token: Token::from("some skill"),
							..default()
						},
					),
					(
						vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right)
						],
						Skill {
							token: Token::from("some child skill"),
							..default()
						},
					),
					(
						vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
						],
						Skill {
							token: Token::from("some right child skill"),
							..default()
						},
					)
				],
				vec![
					(
						vec![SlotKey::BottomHand(Side::Right)],
						Skill {
							token: Token::from("some skill"),
							..default()
						},
					),
					(
						vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right)
						],
						Skill {
							token: Token::from("some child skill"),
							..default()
						},
					),
					(
						vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Left),
						],
						Skill {
							token: Token::from("some left child skill"),
							..default()
						},
					)
				]
			],
			combos.combos_ordered()
		)
	}

	#[test]
	fn get_multiple_combo_with_multiple_deep_child_skills_with_insertion_order_maintained() {
		let combos = ComboNode::new([(
			SlotKey::BottomHand(Side::Right),
			(
				Skill {
					token: Token::from("some skill"),
					..default()
				},
				ComboNode::new([(
					SlotKey::BottomHand(Side::Right),
					(
						Skill {
							token: Token::from("some child skill"),
							..default()
						},
						ComboNode::new([
							(
								SlotKey::BottomHand(Side::Left),
								(
									Skill {
										token: Token::from("some left child skill"),
										..default()
									},
									ComboNode::default(),
								),
							),
							(
								SlotKey::BottomHand(Side::Right),
								(
									Skill {
										token: Token::from("some right child skill"),
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
						Skill {
							token: Token::from("some skill"),
							..default()
						},
					),
					(
						vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right)
						],
						Skill {
							token: Token::from("some child skill"),
							..default()
						},
					),
					(
						vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Left),
						],
						Skill {
							token: Token::from("some left child skill"),
							..default()
						},
					)
				],
				vec![
					(
						vec![SlotKey::BottomHand(Side::Right)],
						Skill {
							token: Token::from("some skill"),
							..default()
						},
					),
					(
						vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right)
						],
						Skill {
							token: Token::from("some child skill"),
							..default()
						},
					),
					(
						vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
						],
						Skill {
							token: Token::from("some right child skill"),
							..default()
						},
					)
				]
			],
			combos.combos_ordered()
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
