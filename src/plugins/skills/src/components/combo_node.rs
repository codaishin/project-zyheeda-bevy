pub(crate) mod dto;

use crate::{
	skills::Skill,
	traits::{combos::UpdateComboSkills, peek_next_recursive::PeekNextRecursive},
};
use bevy::prelude::*;
use common::{
	tools::{
		action_key::slot::SlotKey,
		item_type::{CompatibleItems, ItemType},
		ordered_hash_map::{Entry, IntoIter, OrderedHashMap},
	},
	traits::{
		accessors::get::{GetMut, GetRef},
		handles_loadout::combos::{Combo, GetCombosOrdered, NextConfiguredKeys},
		insert::TryInsert,
		iterate::Iterate,
	},
};
use std::{cmp::Ordering, collections::HashSet};

#[derive(Component, Clone, PartialEq, Debug)]
pub struct ComboNode<TSkill = Skill>(OrderedHashMap<SlotKey, (TSkill, ComboNode<TSkill>)>);

impl<TSkill> ComboNode<TSkill> {
	#[cfg(test)]
	pub fn new<const N: usize>(combos: [(SlotKey, (TSkill, ComboNode<TSkill>)); N]) -> Self {
		Self(OrderedHashMap::from(combos))
	}

	fn get_node(&self, keys: impl Iterator<Item = SlotKey>) -> Option<&Self> {
		let mut node = Some(self);
		for key in keys {
			let Some(Self(children)) = node else {
				return None;
			};

			node = children.get(&key).map(|(_, node)| node);
		}

		node
	}

	fn get_node_mut(&mut self, keys: impl Iterator<Item = SlotKey>) -> Option<&mut Self> {
		let mut node = Some(self);
		for key in keys {
			let Some(Self(children)) = node else {
				return None;
			};

			node = children.get_mut(&key).map(|(_, node)| node);
		}

		node
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

impl<TKey> GetRef<TKey> for ComboNode
where
	for<'a> TKey: Iterate<'a, TItem = &'a SlotKey> + 'a,
{
	type TValue<'a>
		= &'a Skill
	where
		Self: 'a;

	fn get_ref(&self, slot_key_path: &TKey) -> Option<&Skill> {
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

impl<TKey> GetMut<TKey> for ComboNode
where
	for<'a> TKey: Iterate<'a, TItem = &'a SlotKey> + 'a,
{
	type TValue<'a>
		= &'a mut Skill
	where
		Self: 'a;

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

impl<TSkill> NextConfiguredKeys<SlotKey> for ComboNode<TSkill> {
	fn next_keys(&self, combo_keys: &[SlotKey]) -> HashSet<SlotKey> {
		let keys = combo_keys.iter().copied();

		match self.get_node(keys) {
			None => HashSet::default(),
			Some(Self(children)) => children.keys().copied().collect(),
		}
	}
}

impl<TSkill> UpdateComboSkills<TSkill> for ComboNode<TSkill>
where
	TSkill: PartialEq + Clone,
{
	fn update_combo_skills<'a, TComboIter>(&'a mut self, combos: TComboIter)
	where
		TSkill: 'a,
		TComboIter: Iterator<Item = (Vec<SlotKey>, Option<&'a TSkill>)>,
	{
		let mut combos = combos.collect::<Vec<_>>();
		combos.sort_by(key_count_ascending);

		for (keys, skill) in combos {
			let [keys @ .., last] = keys.as_slice() else {
				continue;
			};
			let keys = keys.iter().copied();
			let Some(Self(children)) = self.get_node_mut(keys) else {
				continue;
			};

			match skill {
				Some(skill) if !matches!(children.get(last), Some((s, _)) if s == skill) => {
					children.insert(*last, (skill.clone(), Self::default()));
				}
				None => {
					children.remove(last);
				}
				_ => {}
			};
		}
	}
}

fn key_count_ascending<TSkill>(
	(keys_a, _): &(Vec<SlotKey>, Option<TSkill>),
	(keys_b, _): &(Vec<SlotKey>, Option<TSkill>),
) -> Ordering {
	keys_a.len().cmp(&keys_b.len())
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

impl GetCombosOrdered for ComboNode {
	type TSkill = Skill;

	fn combos_ordered(&self) -> Vec<Combo<SlotKey, Skill>> {
		combos(self, vec![])
	}
}

fn combos(ComboNode(tree): &ComboNode, key_path: Vec<SlotKey>) -> Vec<Combo<SlotKey, Skill>> {
	tree.iter()
		.filter_map(build_path(key_path))
		.flat_map(append_followup_combo_steps)
		.collect()
}

#[allow(clippy::type_complexity)]
fn build_path<'a>(
	key_path: Vec<SlotKey>,
) -> impl FnMut((&SlotKey, &'a (Skill, ComboNode))) -> Option<(Vec<SlotKey>, &'a Skill, &'a ComboNode)>
{
	move |(slot_key, (skill, child_node))| {
		let key_path = [key_path.clone(), vec![*slot_key]].concat();
		Some((key_path, skill, child_node))
	}
}

fn append_followup_combo_steps(
	(key_path, skill, child_node): (Vec<SlotKey>, &Skill, &ComboNode),
) -> Vec<Combo<SlotKey, Skill>> {
	let followup_combo_steps = combos(child_node, key_path.clone());
	append_followups((key_path, skill), followup_combo_steps)
}

fn append_followups<TKey>(
	(combo_path, skill): (Vec<TKey>, &Skill),
	followups: Vec<Combo<TKey, Skill>>,
) -> Vec<Combo<TKey, Skill>>
where
	TKey: TryFrom<SlotKey> + Clone,
{
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
	#![allow(clippy::unwrap_used)]
	use super::*;
	use bevy::prelude::default;
	use common::{tools::action_key::slot::PlayerSlot, traits::handles_localization::Token};
	use std::collections::HashSet;

	#[test]
	fn peek_next_from_tree() {
		let node = ComboNode(OrderedHashMap::from([(
			SlotKey::from(PlayerSlot::LOWER_R),
			(
				Skill {
					token: Token::from("first"),
					compatible_items: CompatibleItems(HashSet::from([ItemType::Pistol])),
					..default()
				},
				ComboNode(OrderedHashMap::from([(
					SlotKey::from(PlayerSlot::LOWER_R),
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

		let next = node.peek_next_recursive(&SlotKey::from(PlayerSlot::LOWER_R), &ItemType::Pistol);

		assert_eq!(
			Some((
				&Skill {
					token: Token::from("first"),
					compatible_items: CompatibleItems(HashSet::from([ItemType::Pistol])),
					..default()
				},
				&ComboNode(OrderedHashMap::from([(
					SlotKey::from(PlayerSlot::LOWER_R),
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
			SlotKey::from(PlayerSlot::LOWER_R),
			(
				Skill {
					token: Token::from("first"),
					compatible_items: CompatibleItems(HashSet::from([ItemType::Bracer])),
					..default()
				},
				ComboNode(OrderedHashMap::from([(
					SlotKey::from(PlayerSlot::LOWER_R),
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

		let next = node.peek_next_recursive(&SlotKey::from(PlayerSlot::LOWER_R), &ItemType::Pistol);

		assert_eq!(None as Option<(&Skill, &ComboNode)>, next)
	}

	#[test]
	fn get_top_skill() {
		let combos = ComboNode::new([(
			SlotKey::from(PlayerSlot::LOWER_R),
			(
				Skill {
					token: Token::from("some skill"),
					..default()
				},
				ComboNode::default(),
			),
		)]);

		let skill = combos.get_ref(&[SlotKey::from(PlayerSlot::LOWER_R)]);

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
			SlotKey::from(PlayerSlot::LOWER_R),
			(
				Skill {
					token: Token::from("some skill"),
					..default()
				},
				ComboNode::new([(
					SlotKey::from(PlayerSlot::LOWER_L),
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

		let skill = combos.get_ref(&[
			SlotKey::from(PlayerSlot::LOWER_R),
			SlotKey::from(PlayerSlot::LOWER_L),
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
			SlotKey::from(PlayerSlot::LOWER_R),
			(
				Skill {
					token: Token::from("some skill"),
					..default()
				},
				ComboNode::default(),
			),
		)]);

		let skill = combos
			.get_mut(&[SlotKey::from(PlayerSlot::LOWER_R)])
			.unwrap();
		*skill = Skill {
			token: Token::from("new skill"),
			..default()
		};

		assert_eq!(
			ComboNode::new([(
				SlotKey::from(PlayerSlot::LOWER_R),
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
			SlotKey::from(PlayerSlot::LOWER_R),
			(
				Skill {
					token: Token::from("some skill"),
					..default()
				},
				ComboNode::new([(
					SlotKey::from(PlayerSlot::LOWER_L),
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
				SlotKey::from(PlayerSlot::LOWER_R),
				SlotKey::from(PlayerSlot::LOWER_L),
			])
			.unwrap();
		*skill = Skill {
			token: Token::from("new skill"),
			..default()
		};

		assert_eq!(
			ComboNode::new([(
				SlotKey::from(PlayerSlot::LOWER_R),
				(
					Skill {
						token: Token::from("some skill"),
						..default()
					},
					ComboNode::new([(
						SlotKey::from(PlayerSlot::LOWER_L),
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
			[SlotKey::from(PlayerSlot::LOWER_R)],
			Skill {
				token: Token::from("new skill"),
				..default()
			},
		);

		assert_eq!(
			(
				ComboNode::new([(
					SlotKey::from(PlayerSlot::LOWER_R),
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
			SlotKey::from(PlayerSlot::LOWER_R),
			(
				Skill {
					token: Token::from("some skill"),
					..default()
				},
				ComboNode::new([(
					SlotKey::from(PlayerSlot::LOWER_R),
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
			[SlotKey::from(PlayerSlot::LOWER_R)],
			Skill {
				token: Token::from("new skill"),
				..default()
			},
		);

		assert_eq!(
			(
				ComboNode::new([(
					SlotKey::from(PlayerSlot::LOWER_R),
					(
						Skill {
							token: Token::from("new skill"),
							..default()
						},
						ComboNode::new([(
							SlotKey::from(PlayerSlot::LOWER_R),
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
			SlotKey::from(PlayerSlot::LOWER_R),
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
				SlotKey::from(PlayerSlot::LOWER_R),
				SlotKey::from(PlayerSlot::LOWER_L),
			],
			Skill {
				token: Token::from("new skill"),
				..default()
			},
		);

		assert_eq!(
			(
				ComboNode::new([(
					SlotKey::from(PlayerSlot::LOWER_R),
					(
						Skill {
							token: Token::from("some skill"),
							..default()
						},
						ComboNode::new([(
							SlotKey::from(PlayerSlot::LOWER_L),
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
				SlotKey::from(PlayerSlot::LOWER_R),
				SlotKey::from(PlayerSlot::LOWER_R),
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
	fn get_single_single_combo_with_single_skill() {
		let combos = ComboNode::new([(
			SlotKey::from(PlayerSlot::LOWER_R),
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
				vec![SlotKey::from(PlayerSlot::LOWER_R)],
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
				SlotKey::from(PlayerSlot::LOWER_R),
				(
					Skill {
						token: Token::from("some right skill"),
						..default()
					},
					ComboNode::default(),
				),
			),
			(
				SlotKey::from(PlayerSlot::LOWER_L),
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
					vec![SlotKey::from(PlayerSlot::LOWER_R)],
					Skill {
						token: Token::from("some right skill"),
						..default()
					},
				)],
				vec![(
					vec![SlotKey::from(PlayerSlot::LOWER_L)],
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
			SlotKey::from(PlayerSlot::LOWER_R),
			(
				Skill {
					token: Token::from("some skill"),
					..default()
				},
				ComboNode::new([(
					SlotKey::from(PlayerSlot::LOWER_L),
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
					vec![SlotKey::from(PlayerSlot::LOWER_R)],
					Skill {
						token: Token::from("some skill"),
						..default()
					},
				),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_L)
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
			SlotKey::from(PlayerSlot::LOWER_R),
			(
				Skill {
					token: Token::from("some skill"),
					..default()
				},
				ComboNode::new([
					(
						SlotKey::from(PlayerSlot::LOWER_R),
						(
							Skill {
								token: Token::from("some right child skill"),
								..default()
							},
							ComboNode::default(),
						),
					),
					(
						SlotKey::from(PlayerSlot::LOWER_L),
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
						vec![SlotKey::from(PlayerSlot::LOWER_R)],
						Skill {
							token: Token::from("some skill"),
							..default()
						},
					),
					(
						vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R)
						],
						Skill {
							token: Token::from("some right child skill"),
							..default()
						},
					)
				],
				vec![
					(
						vec![SlotKey::from(PlayerSlot::LOWER_R)],
						Skill {
							token: Token::from("some skill"),
							..default()
						},
					),
					(
						vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_L)
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
			SlotKey::from(PlayerSlot::LOWER_R),
			(
				Skill {
					token: Token::from("some skill"),
					..default()
				},
				ComboNode::new([(
					SlotKey::from(PlayerSlot::LOWER_R),
					(
						Skill {
							token: Token::from("some child skill"),
							..default()
						},
						ComboNode::new([
							(
								SlotKey::from(PlayerSlot::LOWER_R),
								(
									Skill {
										token: Token::from("some right child skill"),
										..default()
									},
									ComboNode::default(),
								),
							),
							(
								SlotKey::from(PlayerSlot::LOWER_L),
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
						vec![SlotKey::from(PlayerSlot::LOWER_R)],
						Skill {
							token: Token::from("some skill"),
							..default()
						},
					),
					(
						vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R)
						],
						Skill {
							token: Token::from("some child skill"),
							..default()
						},
					),
					(
						vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R),
						],
						Skill {
							token: Token::from("some right child skill"),
							..default()
						},
					)
				],
				vec![
					(
						vec![SlotKey::from(PlayerSlot::LOWER_R)],
						Skill {
							token: Token::from("some skill"),
							..default()
						},
					),
					(
						vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R)
						],
						Skill {
							token: Token::from("some child skill"),
							..default()
						},
					),
					(
						vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_L),
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
			SlotKey::from(PlayerSlot::LOWER_R),
			(
				Skill {
					token: Token::from("some skill"),
					..default()
				},
				ComboNode::new([(
					SlotKey::from(PlayerSlot::LOWER_R),
					(
						Skill {
							token: Token::from("some child skill"),
							..default()
						},
						ComboNode::new([
							(
								SlotKey::from(PlayerSlot::LOWER_L),
								(
									Skill {
										token: Token::from("some left child skill"),
										..default()
									},
									ComboNode::default(),
								),
							),
							(
								SlotKey::from(PlayerSlot::LOWER_R),
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
						vec![SlotKey::from(PlayerSlot::LOWER_R)],
						Skill {
							token: Token::from("some skill"),
							..default()
						},
					),
					(
						vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R)
						],
						Skill {
							token: Token::from("some child skill"),
							..default()
						},
					),
					(
						vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_L),
						],
						Skill {
							token: Token::from("some left child skill"),
							..default()
						},
					)
				],
				vec![
					(
						vec![SlotKey::from(PlayerSlot::LOWER_R)],
						Skill {
							token: Token::from("some skill"),
							..default()
						},
					),
					(
						vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R)
						],
						Skill {
							token: Token::from("some child skill"),
							..default()
						},
					),
					(
						vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R),
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

	mod peek_next {
		use super::*;

		struct _Skill;

		#[test]
		fn get_next() {
			let node = ComboNode::new([(
				SlotKey::from(PlayerSlot::LOWER_R),
				(
					_Skill,
					ComboNode::new([(
						SlotKey::from(PlayerSlot::UPPER_L),
						(
							_Skill,
							ComboNode::new([
								(
									SlotKey::from(PlayerSlot::LOWER_L),
									(_Skill, ComboNode::default()),
								),
								(
									SlotKey::from(PlayerSlot::LOWER_R),
									(_Skill, ComboNode::default()),
								),
							]),
						),
					)]),
				),
			)]);

			assert_eq!(
				HashSet::from([
					SlotKey::from(PlayerSlot::LOWER_L),
					SlotKey::from(PlayerSlot::LOWER_R)
				]),
				node.next_keys(&[
					SlotKey::from(PlayerSlot::LOWER_R),
					SlotKey::from(PlayerSlot::UPPER_L)
				]),
			);
		}
	}

	mod update_combo {
		use super::*;

		#[derive(Debug, PartialEq, Clone)]
		struct _Skill(&'static str);

		#[test]
		fn set_flat_combos() {
			let mut node = ComboNode::new([]);

			node.update_combo_skills(
				[
					(vec![SlotKey::from(PlayerSlot::LOWER_R)], Some(&_Skill("a"))),
					(vec![SlotKey::from(PlayerSlot::LOWER_L)], Some(&_Skill("b"))),
				]
				.into_iter(),
			);

			assert_eq!(
				ComboNode::new([
					(
						SlotKey::from(PlayerSlot::LOWER_R),
						(_Skill("a"), ComboNode::new([]),),
					),
					(
						SlotKey::from(PlayerSlot::LOWER_L),
						(_Skill("b"), ComboNode::new([]),),
					)
				]),
				node,
			);
		}

		#[test]
		fn set_chained_combo() {
			let mut node = ComboNode::new([]);

			node.update_combo_skills(
				[
					(vec![SlotKey::from(PlayerSlot::LOWER_R)], Some(&_Skill("a"))),
					(
						vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_L),
						],
						Some(&_Skill("b")),
					),
				]
				.into_iter(),
			);

			assert_eq!(
				ComboNode::new([(
					SlotKey::from(PlayerSlot::LOWER_R),
					(
						_Skill("a"),
						ComboNode::new([(
							SlotKey::from(PlayerSlot::LOWER_L),
							(_Skill("b"), ComboNode::new([])),
						)]),
					),
				)]),
				node,
			);
		}

		#[test]
		fn set_chained_combo_out_of_order() {
			let mut node = ComboNode::new([]);

			node.update_combo_skills(
				[
					(
						vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_L),
						],
						Some(&_Skill("b")),
					),
					(vec![SlotKey::from(PlayerSlot::LOWER_R)], Some(&_Skill("a"))),
				]
				.into_iter(),
			);

			assert_eq!(
				ComboNode::new([(
					SlotKey::from(PlayerSlot::LOWER_R),
					(
						_Skill("a"),
						ComboNode::new([(
							SlotKey::from(PlayerSlot::LOWER_L),
							(_Skill("b"), ComboNode::new([])),
						)]),
					),
				)]),
				node,
			);
		}

		#[test]
		fn remove_skill_when_input_skill_none() {
			let mut node = ComboNode::new([(
				SlotKey::from(PlayerSlot::LOWER_R),
				(
					_Skill("a"),
					ComboNode::new([
						(
							SlotKey::from(PlayerSlot::LOWER_L),
							(_Skill("b"), ComboNode::new([])),
						),
						(
							SlotKey::from(PlayerSlot::LOWER_R),
							(_Skill("c"), ComboNode::new([])),
						),
					]),
				),
			)]);

			node.update_combo_skills(
				[(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_L),
					],
					None,
				)]
				.into_iter(),
			);

			assert_eq!(
				ComboNode::new([(
					SlotKey::from(PlayerSlot::LOWER_R),
					(
						_Skill("a"),
						ComboNode::new([(
							SlotKey::from(PlayerSlot::LOWER_R),
							(_Skill("c"), ComboNode::new([])),
						)]),
					),
				)]),
				node,
			);
		}

		#[test]
		fn do_not_insert_when_skill_already_behind_key_to_prevent_reordering() {
			let mut node = ComboNode::new([
				(
					SlotKey::from(PlayerSlot::LOWER_R),
					(_Skill("a"), ComboNode::new([])),
				),
				(
					SlotKey::from(PlayerSlot::LOWER_L),
					(_Skill("b"), ComboNode::new([])),
				),
			]);

			node.update_combo_skills(
				[(vec![SlotKey::from(PlayerSlot::LOWER_R)], Some(&_Skill("a")))].into_iter(),
			);

			assert_eq!(
				ComboNode::new([
					(
						SlotKey::from(PlayerSlot::LOWER_R),
						(_Skill("a"), ComboNode::new([])),
					),
					(
						SlotKey::from(PlayerSlot::LOWER_L),
						(_Skill("b"), ComboNode::new([])),
					)
				]),
				node,
			);
		}
	}
}
