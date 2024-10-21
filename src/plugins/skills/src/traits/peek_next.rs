use super::PeekNext;
use crate::{
	components::{combo_node::ComboNode, slots::Slots},
	slot_key::SlotKey,
	skills::Skill,
};

impl<T: PeekNext<(Skill, ComboNode)>> PeekNext<Skill> for T {
	fn peek_next(&self, trigger: &SlotKey, slots: &Slots) -> Option<Skill> {
		self.peek_next(trigger, slots).map(|(skill, _)| skill)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::utils::default;
	use common::components::Side;
	use mockall::{mock, predicate::eq};
	use std::collections::HashMap;

	mock! {
		_Combos {}
		impl PeekNext<(Skill, ComboNode)> for _Combos {
			fn peek_next(&self, trigger: &SlotKey, slots: &Slots) -> Option<(Skill, ComboNode)>;
		}
	}

	fn slots() -> Slots {
		Slots(HashMap::from([(SlotKey::BottomHand(Side::Right), None)]))
	}

	fn other_slots() -> Slots {
		Slots(HashMap::from([(SlotKey::BottomHand(Side::Left), None)]))
	}

	fn node() -> ComboNode {
		ComboNode::new([(
			SlotKey::BottomHand(Side::Right),
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
	fn return_skill() {
		let mut combos = Mock_Combos::default();
		combos.expect_peek_next().return_const((
			Skill {
				name: "my skill".to_owned(),
				..default()
			},
			node(),
		));

		assert_eq!(
			Some(Skill {
				name: "my skill".to_owned(),
				..default()
			}),
			combos.peek_next(&SlotKey::BottomHand(Side::Right), &slots())
		);
	}

	#[test]
	fn return_none() {
		let mut combos = Mock_Combos::default();
		combos.expect_peek_next().return_const(None);

		assert_eq!(
			None as Option<Skill>,
			combos.peek_next(&SlotKey::BottomHand(Side::Right), &slots())
		);
	}

	#[test]
	fn call_peek_next_with_proper_args() {
		let mut combos = Mock_Combos::default();
		combos
			.expect_peek_next()
			.times(1)
			.with(eq(SlotKey::BottomHand(Side::Left)), eq(other_slots()))
			.return_const(None);

		let _: Option<Skill> = combos.peek_next(&SlotKey::BottomHand(Side::Left), &other_slots());
	}
}
