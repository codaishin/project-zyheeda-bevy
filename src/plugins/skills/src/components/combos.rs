pub(crate) mod dto;

use super::combo_node::ComboNode;
use crate::{
	skills::Skill,
	traits::{
		GetNode,
		GetNodeMut,
		Insert,
		ReKey,
		SetNextCombo,
		follow_up_keys::FollowupKeys,
		peek_next::PeekNext,
		peek_next_recursive::PeekNextRecursive,
		write_item::WriteItem,
	},
};
use bevy::ecs::component::Component;
use common::{
	tools::{
		action_key::slot::{Combo, SlotKey},
		item_type::ItemType,
	},
	traits::{handles_combo_menu::GetCombosOrdered, iterate::Iterate},
};
use std::collections::VecDeque;

#[derive(Component, PartialEq, Debug, Clone)]
pub struct Combos<TComboNode = ComboNode> {
	config: TComboNode,
	current: Option<TComboNode>,
}

impl<T> Combos<T> {
	pub fn new(config: T) -> Self {
		Self {
			config,
			current: None,
		}
	}
}

impl<T> From<ComboNode<T>> for Combos<ComboNode<T>> {
	fn from(value: ComboNode<T>) -> Self {
		Combos::new(value)
	}
}

impl Default for Combos {
	fn default() -> Self {
		Self {
			config: ComboNode::default(),
			current: None,
		}
	}
}

impl<TComboNode> SetNextCombo<Option<TComboNode>> for Combos<TComboNode> {
	fn set_next_combo(&mut self, value: Option<TComboNode>) {
		self.current = value;
	}
}

impl<TComboNode> PeekNextRecursive for Combos<TComboNode>
where
	for<'a> TComboNode:
		PeekNextRecursive<TNext<'a> = &'a Skill, TRecursiveNode<'a> = &'a TComboNode> + 'a,
{
	type TNext<'a>
		= &'a Skill
	where
		Self: 'a;
	type TRecursiveNode<'a>
		= &'a TComboNode
	where
		Self: 'a;

	fn peek_next_recursive<'a>(
		&'a self,
		trigger: &SlotKey,
		item_type: &ItemType,
	) -> Option<(Self::TNext<'a>, Self::TRecursiveNode<'a>)> {
		let Combos { config, current } = self;

		current
			.as_ref()
			.and_then(|current| current.peek_next_recursive(trigger, item_type))
			.or_else(|| config.peek_next_recursive(trigger, item_type))
	}
}

impl<'a, TComboNode> PeekNext<'a> for Combos<TComboNode>
where
	Self: PeekNextRecursive<TNext<'a> = &'a Skill, TRecursiveNode<'a> = &'a TComboNode>,
	TComboNode: 'a,
{
	type TNext = Skill;

	fn peek_next(&'a self, trigger: &SlotKey, item_type: &ItemType) -> Option<&'a Skill> {
		self.peek_next_recursive(trigger, item_type)
			.map(|(skill, _)| skill)
	}
}

impl<TNode: GetCombosOrdered<Skill>> GetCombosOrdered<Skill> for Combos<TNode> {
	fn combos_ordered(&self) -> Vec<Combo<Skill>> {
		self.config.combos_ordered()
	}
}

impl<TNode, TKey> WriteItem<TKey, Option<Skill>> for Combos<TNode>
where
	for<'a> TNode: GetNodeMut<TKey, TNode<'a>: Insert<Option<Skill>>>,
	for<'a> TKey: Iterate<'a, TItem = &'a SlotKey> + 'a,
{
	fn write_item(&mut self, key_path: &TKey, skill: Option<Skill>) {
		self.current = None;

		let Some(mut entry) = self.config.node_mut(key_path) else {
			return;
		};

		entry.insert(skill);
	}
}

impl<TNode, TKey> WriteItem<TKey, SlotKey> for Combos<TNode>
where
	for<'a> TNode: GetNodeMut<TKey, TNode<'a>: ReKey<SlotKey>>,
	for<'a> TKey: Iterate<'a, TItem = &'a SlotKey> + 'a,
{
	fn write_item(&mut self, key_path: &TKey, key: SlotKey) {
		self.current = None;

		let Some(mut entry) = self.config.node_mut(key_path) else {
			return;
		};

		entry.re_key(key);
	}
}

impl<TNode, TKey> GetNode<TKey> for Combos<TNode>
where
	TNode: GetNode<TKey>,
	for<'a> TKey: Iterate<'a, TItem = &'a SlotKey> + 'a,
{
	type TNode<'a>
		= TNode::TNode<'a>
	where
		Self: 'a;

	fn node<'a>(&'a self, key: &TKey) -> Option<Self::TNode<'a>> {
		self.config.node(key)
	}
}

impl<TNode> FollowupKeys for Combos<TNode>
where
	TNode: FollowupKeys,
{
	fn followup_keys<T>(&self, after: T) -> Option<Vec<SlotKey>>
	where
		T: Into<VecDeque<SlotKey>>,
	{
		self.config.followup_keys(after)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::utils::default;
	use common::{
		tools::action_key::slot::Side,
		traits::{handles_localization::Token, nested_mock::NestedMocks},
	};
	use macros::NestedMocks;
	use mockall::{mock, predicate::eq};
	use std::{
		cell::RefCell,
		collections::{HashMap, VecDeque},
	};

	#[derive(Default)]
	struct _Next(HashMap<(SlotKey, ItemType), Skill>);

	impl PeekNextRecursive for _Next {
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
		) -> Option<(&'a Skill, &'a Self)> {
			let skill = self.0.get(&(*trigger, *item_type))?;

			Some((skill, self))
		}
	}

	#[test]
	fn return_skill() {
		let trigger = SlotKey::BottomHand(Side::Left);
		let item_type = ItemType::ForceEssence;
		let combos = Combos::new(_Next(HashMap::from([(
			(trigger, item_type),
			Skill {
				token: Token::from("my skill"),
				..default()
			},
		)])));

		let skill = combos
			.peek_next_recursive(&trigger, &item_type)
			.map(|(skill, _)| skill);

		assert_eq!(
			Some(&Skill {
				token: Token::from("my skill"),
				..default()
			}),
			skill
		);
	}

	#[test]
	fn use_combo_used_in_set_next_combo() {
		let trigger = SlotKey::BottomHand(Side::Left);
		let item_type = ItemType::ForceEssence;
		let first = _Next::default();
		let next = _Next(HashMap::from([(
			(trigger, item_type),
			Skill {
				token: Token::from("my skill"),
				..default()
			},
		)]));
		let mut combos = Combos::new(first);
		combos.set_next_combo(Some(next));

		let skill = combos
			.peek_next_recursive(&trigger, &item_type)
			.map(|(skill, _)| skill);

		assert_eq!(
			Some(&Skill {
				token: Token::from("my skill"),
				..default()
			}),
			skill
		);
	}

	#[test]
	fn use_original_when_next_combo_returns_none() {
		let trigger = SlotKey::BottomHand(Side::Left);
		let item_type = ItemType::ForceEssence;
		let first = _Next(HashMap::from([(
			(trigger, item_type),
			Skill {
				token: Token::from("my skill"),
				..default()
			},
		)]));
		let next = _Next::default();
		let mut combos = Combos::new(first);
		combos.set_next_combo(Some(next));

		let skill = combos
			.peek_next_recursive(&trigger, &item_type)
			.map(|(skill, _)| skill);

		assert_eq!(
			Some(&Skill {
				token: Token::from("my skill"),
				..default()
			}),
			skill
		);
	}

	struct _ComboNode(Vec<Combo<Skill>>);

	impl GetCombosOrdered<Skill> for _ComboNode {
		fn combos_ordered(&self) -> Vec<Combo<Skill>> {
			self.0.clone()
		}
	}

	#[test]
	fn get_combos_from_config() {
		let skill = Skill {
			token: Token::from("my skill"),
			..default()
		};
		let combos_vec = vec![vec![(
			vec![
				SlotKey::BottomHand(Side::Left),
				SlotKey::BottomHand(Side::Right),
			],
			skill,
		)]];
		let combos = Combos::new(_ComboNode(combos_vec.clone()));

		assert_eq!(combos_vec, combos.combos_ordered())
	}

	#[derive(Default)]
	struct _Node {
		call_args: RefCell<Vec<Vec<SlotKey>>>,
		entry: Option<_Entry>,
	}

	impl GetNodeMut<Vec<SlotKey>> for _Node {
		type TNode<'a> = &'a mut _Entry;

		fn node_mut<'a>(&'a mut self, key: &Vec<SlotKey>) -> Option<Self::TNode<'a>> {
			self.call_args.get_mut().push(key.clone());
			self.entry.as_mut()
		}
	}

	impl GetNode<Vec<SlotKey>> for _Node {
		type TNode<'a> = &'a _Entry;

		fn node<'a>(&'a self, key: &Vec<SlotKey>) -> Option<Self::TNode<'a>> {
			self.call_args.borrow_mut().push(key.clone());
			self.entry.as_ref()
		}
	}

	mock! {
		_Entry {}
		impl Insert<Option<Skill>> for _Entry {
			fn insert(&mut self, value: Option<Skill>);
		}
		impl ReKey<SlotKey> for _Entry {
			fn re_key(&mut self, key: SlotKey);
		}
	}

	#[derive(NestedMocks)]
	struct _Entry {
		mock: Mock_Entry,
	}

	impl Insert<Option<Skill>> for &mut _Entry {
		fn insert(&mut self, value: Option<Skill>) {
			self.mock.insert(value)
		}
	}

	impl ReKey<SlotKey> for &mut _Entry {
		fn re_key(&mut self, key: SlotKey) {
			self.mock.re_key(key)
		}
	}

	#[test]
	fn update_config_skill_use_correct_arguments() {
		let mut combos = Combos::new(_Node {
			entry: Some(_Entry::new().with_mock(|mock| {
				mock.expect_insert().return_const(());
			})),
			..default()
		});

		combos.write_item(
			&vec![
				SlotKey::BottomHand(Side::Right),
				SlotKey::BottomHand(Side::Left),
			],
			Some(Skill {
				token: Token::from("my skill"),
				..default()
			}),
		);

		assert_eq!(
			vec![vec![
				SlotKey::BottomHand(Side::Right),
				SlotKey::BottomHand(Side::Left)
			]],
			combos.config.call_args.into_inner()
		)
	}

	#[test]
	fn update_config_skill_call_entry_insert() {
		let mut combos = Combos::new(_Node {
			entry: Some(_Entry::new().with_mock(|mock| {
				mock.expect_insert()
					.times(1)
					.with(eq(Some(Skill {
						token: Token::from("my skill"),
						..default()
					})))
					.return_const(());
			})),
			..default()
		});

		combos.write_item(
			&vec![SlotKey::BottomHand(Side::Right)],
			Some(Skill {
				token: Token::from("my skill"),
				..default()
			}),
		);
	}

	#[test]
	fn update_config_skill_clear_current() {
		let mut combos = Combos {
			config: _Node {
				entry: Some(_Entry::new().with_mock(|mock| {
					mock.expect_insert().return_const(());
				})),
				..default()
			},
			current: Some(_Node::default()),
		};

		combos.write_item(
			&vec![SlotKey::BottomHand(Side::Right)],
			Some(Skill {
				token: Token::from("my skill"),
				..default()
			}),
		);

		assert!(combos.current.is_none());
	}

	#[test]
	fn update_config_skill_clear_current_if_entry_is_none() {
		let mut combos = Combos {
			config: _Node {
				entry: None,
				..default()
			},
			current: Some(_Node::default()),
		};

		combos.write_item(
			&vec![SlotKey::BottomHand(Side::Right)],
			Some(Skill {
				token: Token::from("my skill"),
				..default()
			}),
		);

		assert!(combos.current.is_none());
	}

	#[test]
	fn update_config_re_key_use_correct_arguments() {
		let mut combos = Combos::new(_Node {
			entry: Some(_Entry::new().with_mock(|mock| {
				mock.expect_re_key().return_const(());
			})),
			..default()
		});

		combos.write_item(
			&vec![
				SlotKey::BottomHand(Side::Right),
				SlotKey::BottomHand(Side::Left),
			],
			SlotKey::BottomHand(Side::Left),
		);

		assert_eq!(
			vec![vec![
				SlotKey::BottomHand(Side::Right),
				SlotKey::BottomHand(Side::Left)
			]],
			combos.config.call_args.into_inner()
		)
	}

	#[test]
	fn update_config_re_key_call_entry_re_key() {
		let mut combos = Combos::new(_Node {
			entry: Some(_Entry::new().with_mock(|mock| {
				mock.expect_re_key()
					.times(1)
					.with(eq(SlotKey::BottomHand(Side::Left)))
					.return_const(());
			})),
			..default()
		});

		combos.write_item(
			&vec![SlotKey::BottomHand(Side::Right)],
			SlotKey::BottomHand(Side::Left),
		);
	}

	#[test]
	fn update_config_re_key_clear_current() {
		let mut combos = Combos {
			config: _Node {
				entry: Some(_Entry::new().with_mock(|mock| {
					mock.expect_re_key().return_const(());
				})),
				..default()
			},
			current: Some(_Node::default()),
		};

		combos.write_item(
			&vec![SlotKey::BottomHand(Side::Right)],
			SlotKey::BottomHand(Side::Left),
		);

		assert!(combos.current.is_none());
	}

	#[test]
	fn update_config_re_key_clear_current_if_entry_is_none() {
		let mut combos = Combos {
			config: _Node {
				entry: None,
				..default()
			},
			current: Some(_Node::default()),
		};

		combos.write_item(
			&vec![SlotKey::BottomHand(Side::Right)],
			SlotKey::BottomHand(Side::Left),
		);

		assert!(combos.current.is_none());
	}

	#[test]
	fn get_node() {
		let combo = Combos {
			config: _Node {
				entry: Some(_Entry::new().with_mock(|mock| {
					mock.expect_insert().return_const(());
				})),
				..default()
			},
			current: None,
		};

		let entry = combo.node(&vec![
			SlotKey::BottomHand(Side::Right),
			SlotKey::BottomHand(Side::Left),
		]);

		assert_eq!(
			(
				true,
				vec![vec![
					SlotKey::BottomHand(Side::Right),
					SlotKey::BottomHand(Side::Left)
				]]
			),
			(entry.is_some(), combo.config.call_args.into_inner())
		)
	}

	#[test]
	fn get_followup_keys() {
		struct _Node;

		impl FollowupKeys for _Node {
			fn followup_keys<T>(&self, after: T) -> Option<Vec<SlotKey>>
			where
				T: Into<VecDeque<SlotKey>>,
			{
				assert_eq!(VecDeque::from([SlotKey::TopHand(Side::Left)]), after.into());
				Some(vec![SlotKey::TopHand(Side::Right)])
			}
		}

		let combos = Combos::new(_Node);

		assert_eq!(
			Some(vec![SlotKey::TopHand(Side::Right)]),
			combos.followup_keys(vec![SlotKey::TopHand(Side::Left)])
		);
	}
}
