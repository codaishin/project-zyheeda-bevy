pub(crate) mod dto;

use super::combo_node::ComboNode;
use crate::{
	CombosDto,
	skills::Skill,
	traits::{SetNextCombo, peek_next::PeekNext, peek_next_recursive::PeekNextRecursive},
};
use bevy::prelude::*;
use common::{
	tools::{action_key::slot::SlotKey, item_type::ItemType},
	traits::handles_loadout::{
		combos_component::{Combo, GetCombosOrdered, NextConfiguredKeys, UpdateCombos},
		loadout::{LoadoutItem, LoadoutKey},
	},
};
use macros::SavableComponent;
use std::collections::HashSet;

#[derive(Component, SavableComponent, PartialEq, Debug, Clone)]
#[savable_component(dto = CombosDto)]
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

impl<TNode> LoadoutKey for Combos<TNode> {
	type TKey = SlotKey;
}

impl<TNode> LoadoutItem for Combos<TNode> {
	type TItem = Skill;
}

impl<TNode> GetCombosOrdered for Combos<TNode>
where
	TNode: GetCombosOrdered<TKey = SlotKey, TItem = Skill>,
{
	fn combos_ordered(&self) -> Vec<Combo<SlotKey, Skill>> {
		self.config.combos_ordered()
	}
}

impl<TNode> NextConfiguredKeys<SlotKey> for Combos<TNode>
where
	TNode: NextConfiguredKeys<SlotKey>,
{
	fn next_keys(&self, combo_keys: &[SlotKey]) -> HashSet<SlotKey> {
		self.config.next_keys(combo_keys)
	}
}

impl<TNode> UpdateCombos for Combos<TNode>
where
	TNode: UpdateCombos<TKey = SlotKey, TItem = Skill>,
{
	fn update_combos(&mut self, combo: Combo<SlotKey, Option<Skill>>) {
		self.current = None;
		self.config.update_combos(combo);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::utils::default;
	use common::{tools::action_key::slot::PlayerSlot, traits::handles_localization::Token};
	use macros::simple_mock;
	use mockall::predicate::eq;
	use std::collections::HashMap;
	use testing::Mock;

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
		let trigger = SlotKey::from(PlayerSlot::LOWER_L);
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
		let trigger = SlotKey::from(PlayerSlot::LOWER_L);
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
		let trigger = SlotKey::from(PlayerSlot::LOWER_L);
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

	struct _ComboNode(Vec<Combo<SlotKey, Skill>>);

	impl LoadoutKey for _ComboNode {
		type TKey = SlotKey;
	}

	impl LoadoutItem for _ComboNode {
		type TItem = Skill;
	}

	impl GetCombosOrdered for _ComboNode {
		fn combos_ordered(&self) -> Vec<Combo<SlotKey, Skill>> {
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
				SlotKey::from(PlayerSlot::LOWER_L),
				SlotKey::from(PlayerSlot::LOWER_R),
			],
			skill,
		)]];
		let combos = Combos::new(_ComboNode(combos_vec.clone()));

		assert_eq!(combos_vec, combos.combos_ordered())
	}

	mod next_keys {
		use super::*;

		simple_mock! {
			_Node {}
			impl NextConfiguredKeys<SlotKey> for _Node {
				fn next_keys(&self, combo_keys: &[SlotKey]) -> HashSet<SlotKey>;
			}
		}

		#[test]
		fn use_config_node() {
			let combos = Combos::new(Mock_Node::new_mock(|mock| {
				mock.expect_next_keys()
					.times(1)
					.with(eq([
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::UPPER_L),
					]))
					.return_const(HashSet::from([
						SlotKey::from(PlayerSlot::LOWER_L),
						SlotKey::from(PlayerSlot::LOWER_R),
					]));
			}));

			assert_eq!(
				HashSet::from([
					SlotKey::from(PlayerSlot::LOWER_L),
					SlotKey::from(PlayerSlot::LOWER_R)
				]),
				combos.next_keys(&[
					SlotKey::from(PlayerSlot::LOWER_R),
					SlotKey::from(PlayerSlot::UPPER_L)
				]),
			);
		}
	}

	mod update_combo {
		use super::*;

		simple_mock! {
			_Node {}
			impl LoadoutKey for _Node {
				type TKey = SlotKey;
			}
			impl LoadoutItem for _Node {
				type TItem = Skill;
			}
			impl UpdateCombos for _Node {
				fn update_combos(&mut self, combo: Combo<SlotKey, Option<Skill>>);
			}
		}

		#[test]
		fn use_config_node() {
			let mut combos = Combos::new(Mock_Node::new_mock(|mock| {
				mock.expect_update_combos()
					.times(1)
					.with(eq(vec![(
						vec![SlotKey::from(PlayerSlot::LOWER_R)],
						Some(Skill {
							token: Token::from("my skill"),
							..default()
						}),
					)]))
					.return_const(());
			}));

			combos.update_combos(vec![(
				vec![SlotKey::from(PlayerSlot::LOWER_R)],
				Some(Skill {
					token: Token::from("my skill"),
					..default()
				}),
			)]);
		}

		#[test]
		fn reset_current() {
			let mut combos = Combos {
				config: Mock_Node::new_mock(|mock| {
					mock.expect_update_combos().return_const(());
				}),
				current: Some(Mock_Node::new()),
			};

			combos.update_combos(vec![]);

			assert!(combos.current.is_none());
		}
	}
}
