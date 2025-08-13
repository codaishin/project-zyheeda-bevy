pub(crate) mod dto;
pub(crate) mod node_entry_mut;

use crate::{
	skills::Skill,
	traits::{GetNodeMut, peek_next_recursive::PeekNextRecursive},
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
		handles_combo_menu::{Combo, GetCombosOrdered},
		insert::TryInsert,
		iterate::Iterate,
	},
};

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

impl<TKey> GetRef<TKey> for ComboNode
where
	for<'a> TKey: Iterate<'a, TItem = &'a SlotKey> + 'a,
{
	type TValue<'a>
		= &'a Skill
	where
		Self: 'a;

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

#[derive(Debug, PartialEq)]
pub struct NodeEntryMut<'a, TSkill> {
	key: SlotKey,
	tree: &'a mut OrderedHashMap<SlotKey, (TSkill, ComboNode<TSkill>)>,
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
		trigger: SlotKey,
		item_type: &ItemType,
	) -> Option<(Self::TNext<'a>, Self::TRecursiveNode<'a>)> {
		let ComboNode(tree) = self;
		let (skill, combo) = tree.get(&trigger)?;
		let CompatibleItems(is_usable_with) = &skill.compatible_items;

		if !is_usable_with.contains(item_type) {
			return None;
		}

		Some((skill, combo))
	}
}

impl<TKey> GetCombosOrdered<Skill, TKey> for ComboNode
where
	TKey: TryFrom<SlotKey> + Clone,
{
	/// Retrieve configured combos for the given `TKey`
	///
	/// Any slot that does not match `TKey` is dropped in the results.
	fn combos_ordered(&self) -> Vec<Combo<TKey, Skill>> {
		combos(self, vec![])
	}
}

fn combos<TKey>(ComboNode(tree): &ComboNode, key_path: Vec<TKey>) -> Vec<Combo<TKey, Skill>>
where
	TKey: TryFrom<SlotKey> + Clone,
{
	tree.iter()
		.filter_map(build_path(key_path))
		.flat_map(append_followup_combo_steps)
		.collect()
}

#[allow(clippy::type_complexity)]
fn build_path<'a, TKey>(
	key_path: Vec<TKey>,
) -> impl FnMut((&SlotKey, &'a (Skill, ComboNode))) -> Option<(Vec<TKey>, &'a Skill, &'a ComboNode)>
where
	TKey: TryFrom<SlotKey> + Clone,
{
	move |(slot_key, (skill, child_node))| {
		let slot_key = TKey::try_from(*slot_key).ok()?;
		let key_path = [key_path.clone(), vec![slot_key]].concat();
		Some((key_path, skill, child_node))
	}
}

fn append_followup_combo_steps<TKey>(
	(key_path, skill, child_node): (Vec<TKey>, &Skill, &ComboNode),
) -> Vec<Combo<TKey, Skill>>
where
	TKey: TryFrom<SlotKey> + Clone,
{
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
	use super::*;
	use bevy::prelude::default;
	use common::{
		tools::action_key::slot::{PlayerSlot, Side},
		traits::handles_localization::Token,
	};
	use std::collections::HashSet;

	#[test]
	fn peek_next_from_tree() {
		let node = ComboNode(OrderedHashMap::from([(
			SlotKey::from(PlayerSlot::Lower(Side::Right)),
			(
				Skill {
					token: Token::from("first"),
					compatible_items: CompatibleItems(HashSet::from([ItemType::Pistol])),
					..default()
				},
				ComboNode(OrderedHashMap::from([(
					SlotKey::from(PlayerSlot::Lower(Side::Right)),
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

		let next = node.peek_next_recursive(
			SlotKey::from(PlayerSlot::Lower(Side::Right)),
			&ItemType::Pistol,
		);

		assert_eq!(
			Some((
				&Skill {
					token: Token::from("first"),
					compatible_items: CompatibleItems(HashSet::from([ItemType::Pistol])),
					..default()
				},
				&ComboNode(OrderedHashMap::from([(
					SlotKey::from(PlayerSlot::Lower(Side::Right)),
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
			SlotKey::from(PlayerSlot::Lower(Side::Right)),
			(
				Skill {
					token: Token::from("first"),
					compatible_items: CompatibleItems(HashSet::from([ItemType::Bracer])),
					..default()
				},
				ComboNode(OrderedHashMap::from([(
					SlotKey::from(PlayerSlot::Lower(Side::Right)),
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

		let next = node.peek_next_recursive(
			SlotKey::from(PlayerSlot::Lower(Side::Right)),
			&ItemType::Pistol,
		);

		assert_eq!(None as Option<(&Skill, &ComboNode)>, next)
	}

	#[test]
	fn get_top_skill() {
		let combos = ComboNode::new([(
			SlotKey::from(PlayerSlot::Lower(Side::Right)),
			(
				Skill {
					token: Token::from("some skill"),
					..default()
				},
				ComboNode::default(),
			),
		)]);

		let skill = combos.get(&[SlotKey::from(PlayerSlot::Lower(Side::Right))]);

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
			SlotKey::from(PlayerSlot::Lower(Side::Right)),
			(
				Skill {
					token: Token::from("some skill"),
					..default()
				},
				ComboNode::new([(
					SlotKey::from(PlayerSlot::Lower(Side::Left)),
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
			SlotKey::from(PlayerSlot::Lower(Side::Right)),
			SlotKey::from(PlayerSlot::Lower(Side::Left)),
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
			SlotKey::from(PlayerSlot::Lower(Side::Right)),
			(
				Skill {
					token: Token::from("some skill"),
					..default()
				},
				ComboNode::default(),
			),
		)]);

		let skill = combos
			.get_mut(&[SlotKey::from(PlayerSlot::Lower(Side::Right))])
			.unwrap();
		*skill = Skill {
			token: Token::from("new skill"),
			..default()
		};

		assert_eq!(
			ComboNode::new([(
				SlotKey::from(PlayerSlot::Lower(Side::Right)),
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
			SlotKey::from(PlayerSlot::Lower(Side::Right)),
			(
				Skill {
					token: Token::from("some skill"),
					..default()
				},
				ComboNode::new([(
					SlotKey::from(PlayerSlot::Lower(Side::Left)),
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
				SlotKey::from(PlayerSlot::Lower(Side::Right)),
				SlotKey::from(PlayerSlot::Lower(Side::Left)),
			])
			.unwrap();
		*skill = Skill {
			token: Token::from("new skill"),
			..default()
		};

		assert_eq!(
			ComboNode::new([(
				SlotKey::from(PlayerSlot::Lower(Side::Right)),
				(
					Skill {
						token: Token::from("some skill"),
						..default()
					},
					ComboNode::new([(
						SlotKey::from(PlayerSlot::Lower(Side::Left)),
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
			[SlotKey::from(PlayerSlot::Lower(Side::Right))],
			Skill {
				token: Token::from("new skill"),
				..default()
			},
		);

		assert_eq!(
			(
				ComboNode::new([(
					SlotKey::from(PlayerSlot::Lower(Side::Right)),
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
			SlotKey::from(PlayerSlot::Lower(Side::Right)),
			(
				Skill {
					token: Token::from("some skill"),
					..default()
				},
				ComboNode::new([(
					SlotKey::from(PlayerSlot::Lower(Side::Right)),
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
			[SlotKey::from(PlayerSlot::Lower(Side::Right))],
			Skill {
				token: Token::from("new skill"),
				..default()
			},
		);

		assert_eq!(
			(
				ComboNode::new([(
					SlotKey::from(PlayerSlot::Lower(Side::Right)),
					(
						Skill {
							token: Token::from("new skill"),
							..default()
						},
						ComboNode::new([(
							SlotKey::from(PlayerSlot::Lower(Side::Right)),
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
			SlotKey::from(PlayerSlot::Lower(Side::Right)),
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
				SlotKey::from(PlayerSlot::Lower(Side::Right)),
				SlotKey::from(PlayerSlot::Lower(Side::Left)),
			],
			Skill {
				token: Token::from("new skill"),
				..default()
			},
		);

		assert_eq!(
			(
				ComboNode::new([(
					SlotKey::from(PlayerSlot::Lower(Side::Right)),
					(
						Skill {
							token: Token::from("some skill"),
							..default()
						},
						ComboNode::new([(
							SlotKey::from(PlayerSlot::Lower(Side::Left)),
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
				SlotKey::from(PlayerSlot::Lower(Side::Right)),
				SlotKey::from(PlayerSlot::Lower(Side::Right)),
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
			SlotKey::from(PlayerSlot::Lower(Side::Right)),
			(
				Skill {
					token: Token::from("my skill"),
					..default()
				},
				default(),
			),
		)];
		let mut root = ComboNode::new(conf.clone());
		let entry = root.node_mut(&[SlotKey::from(PlayerSlot::Lower(Side::Right))]);

		assert_eq!(
			Some(NodeEntryMut {
				key: SlotKey::from(PlayerSlot::Lower(Side::Right)),
				tree: &mut OrderedHashMap::from(conf),
			}),
			entry,
		)
	}

	#[test]
	fn get_a_mutable_child_entry() {
		let child_conf = [(
			SlotKey::from(PlayerSlot::Lower(Side::Left)),
			(
				Skill {
					token: Token::from("my child skill"),
					..default()
				},
				default(),
			),
		)];
		let conf = [(
			SlotKey::from(PlayerSlot::Lower(Side::Right)),
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
			SlotKey::from(PlayerSlot::Lower(Side::Right)),
			SlotKey::from(PlayerSlot::Lower(Side::Left)),
		]);

		assert_eq!(
			Some(NodeEntryMut {
				key: SlotKey::from(PlayerSlot::Lower(Side::Left)),
				tree: &mut OrderedHashMap::from(child_conf),
			}),
			entry,
		)
	}

	#[test]
	fn get_mutable_none_when_nothing_found_with_key_path() {
		let conf = [(
			SlotKey::from(PlayerSlot::Lower(Side::Right)),
			(
				Skill {
					token: Token::from("my skill"),
					..default()
				},
				ComboNode::new([(
					SlotKey::from(PlayerSlot::Lower(Side::Left)),
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
			SlotKey::from(PlayerSlot::Lower(Side::Right)),
			SlotKey::from(PlayerSlot::Lower(Side::Left)),
			SlotKey::from(PlayerSlot::Lower(Side::Right)),
			SlotKey::from(PlayerSlot::Lower(Side::Left)),
		]);

		assert_eq!(None, entry)
	}

	#[test]
	fn get_a_mutable_entry_when_only_last_in_key_path_not_found() {
		let conf = [(
			SlotKey::from(PlayerSlot::Lower(Side::Right)),
			(
				Skill {
					token: Token::from("my skill"),
					..default()
				},
				ComboNode::new([(
					SlotKey::from(PlayerSlot::Lower(Side::Left)),
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
			SlotKey::from(PlayerSlot::Lower(Side::Right)),
			SlotKey::from(PlayerSlot::Lower(Side::Left)),
			SlotKey::from(PlayerSlot::Lower(Side::Right)),
		]);

		assert_eq!(
			Some(NodeEntryMut {
				key: SlotKey::from(PlayerSlot::Lower(Side::Right)),
				tree: &mut OrderedHashMap::default(),
			}),
			entry,
		)
	}

	#[test]
	fn get_single_single_combo_with_single_skill() {
		let combos = ComboNode::new([(
			SlotKey::from(PlayerSlot::Lower(Side::Right)),
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
				vec![PlayerSlot::Lower(Side::Right)],
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
				SlotKey::from(PlayerSlot::Lower(Side::Right)),
				(
					Skill {
						token: Token::from("some right skill"),
						..default()
					},
					ComboNode::default(),
				),
			),
			(
				SlotKey::from(PlayerSlot::Lower(Side::Left)),
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
					vec![PlayerSlot::Lower(Side::Right)],
					Skill {
						token: Token::from("some right skill"),
						..default()
					},
				)],
				vec![(
					vec![PlayerSlot::Lower(Side::Left)],
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
			SlotKey::from(PlayerSlot::Lower(Side::Right)),
			(
				Skill {
					token: Token::from("some skill"),
					..default()
				},
				ComboNode::new([(
					SlotKey::from(PlayerSlot::Lower(Side::Left)),
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
					vec![PlayerSlot::Lower(Side::Right)],
					Skill {
						token: Token::from("some skill"),
						..default()
					},
				),
				(
					vec![
						PlayerSlot::Lower(Side::Right),
						PlayerSlot::Lower(Side::Left)
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
			SlotKey::from(PlayerSlot::Lower(Side::Right)),
			(
				Skill {
					token: Token::from("some skill"),
					..default()
				},
				ComboNode::new([
					(
						SlotKey::from(PlayerSlot::Lower(Side::Right)),
						(
							Skill {
								token: Token::from("some right child skill"),
								..default()
							},
							ComboNode::default(),
						),
					),
					(
						SlotKey::from(PlayerSlot::Lower(Side::Left)),
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
						vec![PlayerSlot::Lower(Side::Right)],
						Skill {
							token: Token::from("some skill"),
							..default()
						},
					),
					(
						vec![
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Right)
						],
						Skill {
							token: Token::from("some right child skill"),
							..default()
						},
					)
				],
				vec![
					(
						vec![PlayerSlot::Lower(Side::Right)],
						Skill {
							token: Token::from("some skill"),
							..default()
						},
					),
					(
						vec![
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Left)
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
			SlotKey::from(PlayerSlot::Lower(Side::Right)),
			(
				Skill {
					token: Token::from("some skill"),
					..default()
				},
				ComboNode::new([(
					SlotKey::from(PlayerSlot::Lower(Side::Right)),
					(
						Skill {
							token: Token::from("some child skill"),
							..default()
						},
						ComboNode::new([
							(
								SlotKey::from(PlayerSlot::Lower(Side::Right)),
								(
									Skill {
										token: Token::from("some right child skill"),
										..default()
									},
									ComboNode::default(),
								),
							),
							(
								SlotKey::from(PlayerSlot::Lower(Side::Left)),
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
						vec![PlayerSlot::Lower(Side::Right)],
						Skill {
							token: Token::from("some skill"),
							..default()
						},
					),
					(
						vec![
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Right)
						],
						Skill {
							token: Token::from("some child skill"),
							..default()
						},
					),
					(
						vec![
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Right),
						],
						Skill {
							token: Token::from("some right child skill"),
							..default()
						},
					)
				],
				vec![
					(
						vec![PlayerSlot::Lower(Side::Right)],
						Skill {
							token: Token::from("some skill"),
							..default()
						},
					),
					(
						vec![
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Right)
						],
						Skill {
							token: Token::from("some child skill"),
							..default()
						},
					),
					(
						vec![
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Left),
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
			SlotKey::from(PlayerSlot::Lower(Side::Right)),
			(
				Skill {
					token: Token::from("some skill"),
					..default()
				},
				ComboNode::new([(
					SlotKey::from(PlayerSlot::Lower(Side::Right)),
					(
						Skill {
							token: Token::from("some child skill"),
							..default()
						},
						ComboNode::new([
							(
								SlotKey::from(PlayerSlot::Lower(Side::Left)),
								(
									Skill {
										token: Token::from("some left child skill"),
										..default()
									},
									ComboNode::default(),
								),
							),
							(
								SlotKey::from(PlayerSlot::Lower(Side::Right)),
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
						vec![PlayerSlot::Lower(Side::Right)],
						Skill {
							token: Token::from("some skill"),
							..default()
						},
					),
					(
						vec![
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Right)
						],
						Skill {
							token: Token::from("some child skill"),
							..default()
						},
					),
					(
						vec![
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Left),
						],
						Skill {
							token: Token::from("some left child skill"),
							..default()
						},
					)
				],
				vec![
					(
						vec![PlayerSlot::Lower(Side::Right)],
						Skill {
							token: Token::from("some skill"),
							..default()
						},
					),
					(
						vec![
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Right)
						],
						Skill {
							token: Token::from("some child skill"),
							..default()
						},
					),
					(
						vec![
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Right),
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
}
