use super::combo_node::ComboNode;
use crate::{
	item::item_type::SkillItemType,
	skills::Skill,
	slot_key::SlotKey,
	traits::{
		Combo,
		GetCombosOrdered,
		GetNode,
		GetNodeMut,
		Insert,
		PeekNext,
		ReKey,
		RootKeys,
		SetNextCombo,
		UpdateConfig,
	},
};
use bevy::ecs::component::Component;
use common::traits::iterate::Iterate;

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

impl<TComboNode> PeekNext<(Skill, TComboNode)> for Combos<TComboNode>
where
	TComboNode: PeekNext<(Skill, TComboNode)>,
{
	fn peek_next(
		&self,
		trigger: &SlotKey,
		item_type: &SkillItemType,
	) -> Option<(Skill, TComboNode)> {
		let Combos { config, current } = self;

		current
			.as_ref()
			.and_then(|current| current.peek_next(trigger, item_type))
			.or_else(|| config.peek_next(trigger, item_type))
	}
}

impl<TNode: GetCombosOrdered> GetCombosOrdered for Combos<TNode> {
	fn combos_ordered(&self) -> impl Iterator<Item = Combo> {
		self.config.combos_ordered()
	}
}

impl<TNode, TKey> UpdateConfig<TKey, Option<Skill>> for Combos<TNode>
where
	for<'a> TNode: GetNodeMut<'a, TKey, TNode: Insert<Option<Skill>>>,
	TKey: Iterate<SlotKey>,
{
	fn update_config(&mut self, key_path: &TKey, skill: Option<Skill>) {
		self.current = None;

		let Some(mut entry) = self.config.node_mut(key_path) else {
			return;
		};

		entry.insert(skill);
	}
}

impl<TNode, TKey> UpdateConfig<TKey, SlotKey> for Combos<TNode>
where
	for<'a> TNode: GetNodeMut<'a, TKey, TNode: ReKey<SlotKey>>,
	TKey: Iterate<SlotKey>,
{
	fn update_config(&mut self, key_path: &TKey, key: SlotKey) {
		self.current = None;

		let Some(mut entry) = self.config.node_mut(key_path) else {
			return;
		};

		entry.re_key(key);
	}
}

impl<'a, TNode: GetNode<'a, TKey>, TKey: Iterate<SlotKey>> GetNode<'a, TKey> for Combos<TNode> {
	type TNode = TNode::TNode;

	fn node(&'a self, key: &TKey) -> Option<Self::TNode> {
		self.config.node(key)
	}
}

impl<TNode: RootKeys> RootKeys for Combos<TNode> {
	type TItem = TNode::TItem;

	fn root_keys(&self) -> impl Iterator<Item = Self::TItem> {
		self.config.root_keys()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::utils::default;
	use common::{
		components::Side,
		simple_init,
		traits::{mock::Mock, nested_mock::NestedMocks},
	};
	use macros::NestedMocks;
	use mockall::{mock, predicate::eq};
	use std::cell::RefCell;

	mock! {
		_Next {}
		impl PeekNext<(Skill, Self)> for _Next {
			fn peek_next(&self, _trigger: &SlotKey, _item_type: &SkillItemType) -> Option<(Skill, Self)>;
		}
	}

	simple_init!(Mock_Next);

	#[test]
	fn call_next_with_correct_args() {
		let item_type = SkillItemType::ForceEssence;
		let trigger = SlotKey::BottomHand(Side::Left);
		let combos = Combos::new(Mock_Next::new_mock(|mock| {
			mock.expect_peek_next()
				.times(1)
				.with(eq(trigger), eq(item_type))
				.returning(|_, _| None);
		}));

		combos.peek_next(&trigger, &item_type);
	}

	#[test]
	fn return_skill() {
		let combos = Combos::new(Mock_Next::new_mock(|mock| {
			mock.expect_peek_next().returning(|_, _| {
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
			.peek_next(&SlotKey::default(), &SkillItemType::default())
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
			mock.expect_peek_next().never().returning(|_, _| None);
		});
		let next = Mock_Next::new_mock(|mock| {
			mock.expect_peek_next()
				.with(eq(SlotKey::TopHand(Side::Left)), eq(SkillItemType::Pistol))
				.times(1)
				.returning(|_, _| Some((Skill::default(), Mock_Next::default())));
		});
		let mut combos = Combos::new(first);
		combos.set_next_combo(Some(next));

		combos.peek_next(&SlotKey::TopHand(Side::Left), &SkillItemType::Pistol);
	}

	#[test]
	fn use_original_when_next_combo_returns_none() {
		let first = Mock_Next::new_mock(|mock| {
			mock.expect_peek_next().times(1).returning(|_, _| None);
		});
		let next = Mock_Next::new_mock(|mock| {
			mock.expect_peek_next().returning(|_, _| None);
		});
		let mut combos = Combos::new(first);
		combos.set_next_combo(Some(next));

		combos.peek_next(&SlotKey::default(), &SkillItemType::default());
	}

	struct _ComboNode<'a>(Vec<Combo<'a>>);

	impl<'a> GetCombosOrdered for _ComboNode<'a> {
		fn combos_ordered(&self) -> impl Iterator<Item = Combo> {
			self.0.iter().cloned()
		}
	}

	#[test]
	fn get_combos_from_config() {
		let skill = Skill {
			name: "my skill".to_owned(),
			..default()
		};
		let combos_vec = vec![vec![(
			vec![
				SlotKey::BottomHand(Side::Left),
				SlotKey::BottomHand(Side::Right),
			],
			&skill,
		)]];
		let combos = Combos::new(_ComboNode(combos_vec.clone()));

		assert_eq!(combos_vec, combos.combos_ordered().collect::<Vec<_>>())
	}

	#[derive(Default)]
	struct _Node {
		call_args: RefCell<Vec<Vec<SlotKey>>>,
		entry: Option<_Entry>,
	}

	impl<'a> GetNodeMut<'a, Vec<SlotKey>> for _Node {
		type TNode = &'a mut _Entry;

		fn node_mut(&'a mut self, key: &Vec<SlotKey>) -> Option<Self::TNode> {
			self.call_args.get_mut().push(key.clone());
			self.entry.as_mut()
		}
	}

	impl<'a> GetNode<'a, Vec<SlotKey>> for _Node {
		type TNode = &'a _Entry;

		fn node(&'a self, key: &Vec<SlotKey>) -> Option<Self::TNode> {
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

		combos.update_config(
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

		combos.update_config(
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

		combos.update_config(
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

		combos.update_config(
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

		combos.update_config(
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

		combos.update_config(
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

		combos.update_config(
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

		combos.update_config(
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
}
