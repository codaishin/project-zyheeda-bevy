use std::collections::VecDeque;

use super::combo_node::ComboNode;
use crate::{
	skills::Skill,
	traits::{
		peek_next_recursive::PeekNextRecursive,
		GetNode,
		GetNodeMut,
		Insert,
		ReKey,
		RootKeys,
		SetNextCombo,
	},
};
use bevy::ecs::component::Component;
use common::{
	tools::{item_type::ItemType, slot_key::SlotKey},
	traits::{
		handles_combo_menu::{ComboSkillDescriptor, GetCombosOrdered},
		handles_equipment::{Combo, FollowupKeys, PeekNext, WriteItem},
		iterate::Iterate,
	},
};

#[derive(Component, PartialEq, Debug)]
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
	TComboNode: PeekNextRecursive<TNext = Skill, TRecursiveNode = TComboNode>,
{
	type TNext = Skill;
	type TRecursiveNode = TComboNode;

	fn peek_next_recursive(
		&self,
		trigger: &SlotKey,
		item_type: &ItemType,
	) -> Option<(Self::TNext, Self::TRecursiveNode)> {
		let Combos { config, current } = self;

		current
			.as_ref()
			.and_then(|current| current.peek_next_recursive(trigger, item_type))
			.or_else(|| config.peek_next_recursive(trigger, item_type))
	}
}

impl<TComboNode> PeekNext for Combos<TComboNode>
where
	Self: PeekNextRecursive<TNext = Skill, TRecursiveNode = TComboNode>,
{
	type TNext = Skill;

	fn peek_next(&self, trigger: &SlotKey, item_type: &ItemType) -> Option<Skill> {
		self.peek_next_recursive(trigger, item_type)
			.map(|(skill, _)| skill)
	}
}

impl<TNode: GetCombosOrdered<Skill>> GetCombosOrdered<Skill> for Combos<TNode> {
	fn combos_ordered(&self) -> Vec<Combo<ComboSkillDescriptor<Skill>>> {
		self.config.combos_ordered()
	}
}

impl<TNode, TKey> WriteItem<TKey, Option<Skill>> for Combos<TNode>
where
	for<'a> TNode: GetNodeMut<TKey, TNode<'a>: Insert<Option<Skill>>>,
	TKey: Iterate<SlotKey>,
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
	TKey: Iterate<SlotKey>,
{
	fn write_item(&mut self, key_path: &TKey, key: SlotKey) {
		self.current = None;

		let Some(mut entry) = self.config.node_mut(key_path) else {
			return;
		};

		entry.re_key(key);
	}
}

impl<TNode: GetNode<TKey>, TKey: Iterate<SlotKey>> GetNode<TKey> for Combos<TNode> {
	type TNode<'a>
		= TNode::TNode<'a>
	where
		Self: 'a;

	fn node<'a>(&'a self, key: &TKey) -> Option<Self::TNode<'a>> {
		self.config.node(key)
	}
}

impl<TNode: RootKeys> RootKeys for Combos<TNode> {
	type TItem = TNode::TItem;

	fn root_keys(&self) -> impl Iterator<Item = Self::TItem> {
		self.config.root_keys()
	}
}

impl<TNode> FollowupKeys for Combos<TNode>
where
	TNode: FollowupKeys,
{
	type TKey = TNode::TKey;

	fn followup_keys<T>(&self, after: T) -> Option<Vec<Self::TKey>>
	where
		T: Into<VecDeque<Self::TKey>> + 'static,
	{
		self.config.followup_keys(after)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::utils::default;
	use common::{
		simple_init,
		tools::slot_key::Side,
		traits::{mock::Mock, nested_mock::NestedMocks},
	};
	use macros::NestedMocks;
	use mockall::{mock, predicate::eq};
	use std::{cell::RefCell, collections::VecDeque};

	mock! {
		_Next {}
		impl PeekNextRecursive for _Next {
			type TNext = Skill;
			type TRecursiveNode = Self;

			fn peek_next_recursive(&self, _trigger: &SlotKey, _item_type: &ItemType) -> Option<(Skill, Self)>;
		}
	}

	simple_init!(Mock_Next);

	#[test]
	fn call_next_with_correct_args() {
		let item_type = ItemType::ForceEssence;
		let trigger = SlotKey::BottomHand(Side::Left);
		let combos = Combos::new(Mock_Next::new_mock(|mock| {
			mock.expect_peek_next_recursive()
				.times(1)
				.with(eq(trigger), eq(item_type))
				.returning(|_, _| None);
		}));

		combos.peek_next_recursive(&trigger, &item_type);
	}

	#[test]
	fn return_skill() {
		let combos = Combos::new(Mock_Next::new_mock(|mock| {
			mock.expect_peek_next_recursive().returning(|_, _| {
				Some((
					Skill {
						name: "my skill".to_owned(),
						..default()
					},
					Mock_Next::default(),
				))
			});
		}));

		let skill = combos
			.peek_next_recursive(&SlotKey::default(), &ItemType::default())
			.map(|(skill, _)| skill);

		assert_eq!(
			Some(Skill {
				name: "my skill".to_owned(),
				..default()
			}),
			skill
		);
	}

	#[test]
	fn use_combo_used_in_set_next_combo() {
		let first = Mock_Next::new_mock(|mock| {
			mock.expect_peek_next_recursive()
				.never()
				.returning(|_, _| None);
		});
		let next = Mock_Next::new_mock(|mock| {
			mock.expect_peek_next_recursive()
				.with(eq(SlotKey::TopHand(Side::Left)), eq(ItemType::Pistol))
				.times(1)
				.returning(|_, _| Some((Skill::default(), Mock_Next::default())));
		});
		let mut combos = Combos::new(first);
		combos.set_next_combo(Some(next));

		combos.peek_next_recursive(&SlotKey::TopHand(Side::Left), &ItemType::Pistol);
	}

	#[test]
	fn use_original_when_next_combo_returns_none() {
		let first = Mock_Next::new_mock(|mock| {
			mock.expect_peek_next_recursive()
				.times(1)
				.returning(|_, _| None);
		});
		let next = Mock_Next::new_mock(|mock| {
			mock.expect_peek_next_recursive().returning(|_, _| None);
		});
		let mut combos = Combos::new(first);
		combos.set_next_combo(Some(next));

		combos.peek_next_recursive(&SlotKey::default(), &ItemType::default());
	}

	struct _ComboNode(Vec<Combo<ComboSkillDescriptor<Skill>>>);

	impl GetCombosOrdered<Skill> for _ComboNode {
		fn combos_ordered(&self) -> Vec<Combo<ComboSkillDescriptor<Skill>>> {
			self.0.clone()
		}
	}

	#[test]
	fn get_combos_from_config() {
		let skill = ComboSkillDescriptor::<Skill> {
			name: "my skill".to_owned(),
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
				name: "my skill".to_owned(),
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
						name: "my skill".to_owned(),
						..default()
					})))
					.return_const(());
			})),
			..default()
		});

		combos.write_item(
			&vec![SlotKey::BottomHand(Side::Right)],
			Some(Skill {
				name: "my skill".to_owned(),
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
				name: "my skill".to_owned(),
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
				name: "my skill".to_owned(),
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
	fn get_root_keys() {
		#[derive(Debug, PartialEq)]
		struct _Key;

		struct _Node;

		impl RootKeys for _Node {
			type TItem = _Key;

			fn root_keys(&self) -> impl Iterator<Item = Self::TItem> {
				vec![_Key].into_iter()
			}
		}

		let combos = Combos::new(_Node);

		assert_eq!(vec![_Key], combos.root_keys().collect::<Vec<_>>());
	}

	#[test]
	fn get_followup_keys() {
		#[derive(Debug, PartialEq)]
		struct _Key;

		struct _Node;

		impl FollowupKeys for _Node {
			type TKey = _Key;

			fn followup_keys<T>(&self, after: T) -> Option<Vec<Self::TKey>>
			where
				T: Into<VecDeque<Self::TKey>>,
			{
				assert_eq!(VecDeque::from([_Key]), after.into());
				Some(vec![_Key])
			}
		}

		let combos = Combos::new(_Node);

		assert_eq!(Some(vec![_Key]), combos.followup_keys(vec![_Key]));
	}
}
