use super::{AdvanceCombo, SetNextCombo, peek_next_recursive::PeekNextRecursive};
use crate::{components::combo_node::ComboNode, skills::Skill};
use common::tools::{action_key::slot::SlotKey, item_type::ItemType};

impl<T> AdvanceCombo for T
where
	for<'a> T: PeekNextRecursive<TNext<'a> = &'a Skill, TRecursiveNode<'a> = &'a ComboNode>
		+ SetNextCombo<Option<ComboNode>>
		+ 'a,
{
	fn advance_combo(&mut self, trigger: SlotKey, item_type: &ItemType) -> Option<Skill> {
		let Some((skill, next_combo)) = self.peek_next_recursive(trigger, item_type) else {
			self.set_next_combo(None);
			return None;
		};
		let next = next_combo.clone();
		let skill = skill.clone();

		self.set_next_combo(Some(next));
		Some(skill)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::utils::default;
	use common::{tools::action_key::slot::SlotKey, traits::handles_localization::Token};
	use macros::simple_mock;
	use mockall::predicate::eq;
	use std::collections::HashMap;

	struct _Combos {
		lookup: HashMap<(SlotKey, ItemType), Skill>,
		mock: Mock_Combos,
		node: ComboNode,
	}

	impl _Combos {
		fn new<const N: usize>(
			node: ComboNode,
			lookup: [((SlotKey, ItemType), Skill); N],
			setup_mock: impl Fn(&mut Mock_Combos),
		) -> Self {
			let mut mock = Mock_Combos::default();
			setup_mock(&mut mock);

			Self {
				lookup: HashMap::from(lookup),
				mock,
				node,
			}
		}
	}

	impl PeekNextRecursive for _Combos {
		type TNext<'a> = &'a Skill;
		type TRecursiveNode<'a> = &'a ComboNode;

		fn peek_next_recursive<'a>(
			&'a self,
			trigger: SlotKey,
			item_type: &ItemType,
		) -> Option<(&'a Skill, &'a ComboNode)> {
			let skill = self.lookup.get(&(trigger, *item_type))?;

			Some((skill, &self.node))
		}
	}

	impl SetNextCombo<Option<ComboNode>> for _Combos {
		fn set_next_combo(&mut self, value: Option<ComboNode>) {
			self.mock.set_next_combo(value);
		}
	}

	simple_mock! {
		_Combos {}
		impl SetNextCombo<Option<ComboNode>> for _Combos {
			fn set_next_combo(&mut self, value: Option<ComboNode>);
		}
	}

	fn node() -> ComboNode {
		ComboNode::new([(
			SlotKey(11),
			(
				Skill {
					token: Token::from("some skill"),
					..default()
				},
				ComboNode::default(),
			),
		)])
	}

	#[test]
	fn call_set_next_combo_with_next_when_peek_was_some() {
		let mut combos = _Combos::new(
			node(),
			[((SlotKey(11), ItemType::Pistol), Skill::default())],
			|mock| {
				mock.expect_set_next_combo()
					.times(1)
					.with(eq(Some(node())))
					.return_const(());
			},
		);

		combos.advance_combo(SlotKey(11), &ItemType::Pistol);
	}

	#[test]
	fn return_skill_when_peek_next_was_some() {
		let mut combos = _Combos::new(
			node(),
			[(
				(SlotKey(0), ItemType::default()),
				Skill {
					token: Token::from("return this"),
					..default()
				},
			)],
			|mock| {
				mock.expect_set_next_combo().return_const(());
			},
		);

		let skill = combos.advance_combo(SlotKey(0), &ItemType::default());

		assert_eq!(
			Some(Skill {
				token: Token::from("return this"),
				..default()
			}),
			skill
		);
	}

	#[test]
	fn call_set_next_combo_with_none_when_peek_was_none() {
		let mut combos = _Combos::new(node(), [], |mock| {
			mock.expect_set_next_combo()
				.times(1)
				.with(eq(None))
				.return_const(());
		});

		combos.advance_combo(SlotKey(0), &ItemType::default());
	}

	#[test]
	fn return_none_when_peek_next_was_none() {
		let mut combos = _Combos::new(node(), [], |mock| {
			mock.expect_set_next_combo().return_const(());
		});

		let skill = combos.advance_combo(SlotKey(0), &ItemType::default());

		assert_eq!(None, skill);
	}
}
