use super::{AdvanceCombo, PeekNext, SetNextCombo};
use crate::{
	components::{combo_node::ComboNode, slots::Slots},
	items::slot_key::SlotKey,
	skills::Skill,
};

impl<T: PeekNext<(Skill, ComboNode)> + SetNextCombo<Option<ComboNode>>> AdvanceCombo for T {
	fn advance(&mut self, trigger: &SlotKey, slots: &Slots) -> Option<Skill> {
		let Some((skill, next_combo)) = self.peek_next(trigger, slots) else {
			self.set_next_combo(None);
			return None;
		};
		self.set_next_combo(Some(next_combo));
		Some(skill)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{Mounts, Slot};
	use bevy::{ecs::entity::Entity, utils::default};
	use common::components::Side;
	use mockall::{mock, predicate::eq};
	use std::collections::HashMap;

	mock! {
		_Combos {}
		impl PeekNext<(Skill, ComboNode)> for _Combos {
			fn peek_next(&self, trigger: &SlotKey, slots: &Slots) -> Option<(Skill, ComboNode)>;
		}
		impl SetNextCombo<Option<ComboNode>> for _Combos {
			fn set_next_combo(&mut self, value: Option<ComboNode>);
		}
	}

	fn slots() -> Slots {
		Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				mounts: Mounts {
					hand: Entity::from_raw(123),
					forearm: Entity::from_raw(456),
				},
				item: None,
			},
		)]))
	}

	fn other_slots() -> Slots {
		Slots(HashMap::from([(
			SlotKey::Hand(Side::Off),
			Slot {
				mounts: Mounts {
					hand: Entity::from_raw(321),
					forearm: Entity::from_raw(654),
				},
				item: None,
			},
		)]))
	}

	fn node() -> ComboNode {
		ComboNode::new([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "some skill".to_owned(),
					..default()
				},
				ComboNode::default(),
			),
		)])
	}

	#[test]
	fn call_set_next_combo_with_next_when_peek_was_some() {
		let mut combos = Mock_Combos::default();
		combos
			.expect_peek_next()
			.return_const((Skill::default(), node()));
		combos
			.expect_set_next_combo()
			.times(1)
			.with(eq(Some(node())))
			.return_const(());

		combos.advance(&SlotKey::Hand(Side::Main), &slots());
	}

	#[test]
	fn return_skill_when_peek_next_was_some() {
		let mut combos = Mock_Combos::default();
		combos.expect_peek_next().return_const((
			Skill {
				name: "return this".to_owned(),
				..default()
			},
			node(),
		));
		combos.expect_set_next_combo().return_const(());

		let skill = combos.advance(&SlotKey::Hand(Side::Main), &slots());

		assert_eq!(
			Some(Skill {
				name: "return this".to_owned(),
				..default()
			}),
			skill
		);
	}

	#[test]
	fn call_set_next_combo_with_none_when_peek_was_none() {
		let mut combos = Mock_Combos::default();
		combos.expect_peek_next().return_const(None);
		combos
			.expect_set_next_combo()
			.times(1)
			.with(eq(None))
			.return_const(());

		combos.advance(&SlotKey::Hand(Side::Main), &slots());
	}

	#[test]
	fn return_none_when_peek_next_was_none() {
		let mut combos = Mock_Combos::default();
		combos.expect_peek_next().return_const(None);
		combos.expect_set_next_combo().return_const(());

		let skill = combos.advance(&SlotKey::Hand(Side::Main), &slots());

		assert_eq!(None, skill);
	}

	#[test]
	fn call_peek_next_with_correct_args() {
		let mut combos = Mock_Combos::default();
		combos
			.expect_peek_next()
			.times(1)
			.with(eq(SlotKey::Hand(Side::Off)), eq(other_slots()))
			.return_const(None);
		combos.expect_set_next_combo().return_const(());

		combos.advance(&SlotKey::Hand(Side::Off), &other_slots());
	}
}
