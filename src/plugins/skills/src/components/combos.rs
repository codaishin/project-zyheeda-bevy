pub(crate) mod dto;

use super::combo_node::ComboNode;
use crate::{
	CombosDto,
	skills::Skill,
	traits::{SetNextCombo, peek_next::PeekNext, peek_next_recursive::PeekNextRecursive},
};
use bevy::prelude::*;
use common::{
	tools::{
		action_key::slot::{PlayerSlot, SlotKey},
		item_type::ItemType,
	},
	traits::{
		handles_combo_menu::{Combo, GetCombosOrdered, NextConfiguredKeys},
		handles_loadout::UpdateCombos,
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
		trigger: SlotKey,
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

	fn peek_next(&'a self, trigger: SlotKey, item_type: &ItemType) -> Option<&'a Skill> {
		self.peek_next_recursive(trigger, item_type)
			.map(|(skill, _)| skill)
	}
}

impl<TNode, TKey> GetCombosOrdered<Skill, TKey> for Combos<TNode>
where
	TNode: GetCombosOrdered<Skill, TKey>,
{
	fn combos_ordered(&self) -> Vec<Combo<TKey, Skill>> {
		self.config.combos_ordered()
	}
}

impl<TNode> NextConfiguredKeys<PlayerSlot> for Combos<TNode>
where
	TNode: NextConfiguredKeys<PlayerSlot>,
{
	fn next_keys(&self, combo_keys: &[PlayerSlot]) -> HashSet<PlayerSlot> {
		self.config.next_keys(combo_keys)
	}
}

impl<TNode, TSkill> UpdateCombos<TSkill> for Combos<TNode>
where
	TNode: UpdateCombos<TSkill>,
{
	fn update_combos(&mut self, combo: Combo<PlayerSlot, Option<TSkill>>) {
		self.current = None;
		self.config.update_combos(combo);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::utils::default;
	use common::{
		tools::action_key::slot::{PlayerSlot, Side},
		traits::handles_localization::Token,
	};
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
			trigger: SlotKey,
			item_type: &ItemType,
		) -> Option<(&'a Skill, &'a Self)> {
			let skill = self.0.get(&(trigger, *item_type))?;

			Some((skill, self))
		}
	}

	#[test]
	fn return_skill() {
		let trigger = SlotKey::from(PlayerSlot::Lower(Side::Left));
		let item_type = ItemType::ForceEssence;
		let combos = Combos::new(_Next(HashMap::from([(
			(trigger, item_type),
			Skill {
				token: Token::from("my skill"),
				..default()
			},
		)])));

		let skill = combos
			.peek_next_recursive(trigger, &item_type)
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
		let trigger = SlotKey::from(PlayerSlot::Lower(Side::Left));
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
			.peek_next_recursive(trigger, &item_type)
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
		let trigger = SlotKey::from(PlayerSlot::Lower(Side::Left));
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
			.peek_next_recursive(trigger, &item_type)
			.map(|(skill, _)| skill);

		assert_eq!(
			Some(&Skill {
				token: Token::from("my skill"),
				..default()
			}),
			skill
		);
	}

	struct _ComboNode(Vec<Combo<PlayerSlot, Skill>>);

	impl GetCombosOrdered<Skill, PlayerSlot> for _ComboNode {
		fn combos_ordered(&self) -> Vec<Combo<PlayerSlot, Skill>> {
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
				PlayerSlot::Lower(Side::Left),
				PlayerSlot::Lower(Side::Right),
			],
			skill,
		)]];
		let combos = Combos::new(_ComboNode(combos_vec.clone()));

		assert_eq!(combos_vec, combos.combos_ordered())
	}

	mod next_keys {
		use super::*;

		struct _Skill;

		simple_mock! {
			_Node {}
			impl NextConfiguredKeys<PlayerSlot> for _Node {
				fn next_keys(&self, combo_keys: &[PlayerSlot]) -> HashSet<PlayerSlot>;
			}
		}

		#[test]
		fn use_config_node() {
			let combos = Combos::new(Mock_Node::new_mock(|mock| {
				mock.expect_next_keys()
					.times(1)
					.with(eq([PlayerSlot::LOWER_R, PlayerSlot::UPPER_L]))
					.return_const(HashSet::from([PlayerSlot::LOWER_L, PlayerSlot::LOWER_R]));
			}));

			assert_eq!(
				HashSet::from([PlayerSlot::LOWER_L, PlayerSlot::LOWER_R]),
				combos.next_keys(&[PlayerSlot::LOWER_R, PlayerSlot::UPPER_L]),
			);
		}
	}

	mod update_combo {
		use super::*;

		#[derive(Debug, PartialEq)]
		struct _Skill;

		simple_mock! {
			_Node {}
			impl UpdateCombos<_Skill> for _Node {
				fn update_combos(&mut self, combo: Combo<PlayerSlot, Option<_Skill>>);
			}
		}

		#[test]
		fn use_config_node() {
			let mut combos = Combos::new(Mock_Node::new_mock(|mock| {
				mock.expect_update_combos()
					.times(1)
					.with(eq(vec![(vec![PlayerSlot::LOWER_R], Some(_Skill))]))
					.return_const(());
			}));

			combos.update_combos(vec![(vec![PlayerSlot::LOWER_R], Some(_Skill))]);
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
