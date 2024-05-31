use super::PeekNext;
use crate::{
	components::{combos::ComboNode, slots::Slots},
	items::SlotKey,
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
		ComboNode::Tree(HashMap::from([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "some skill",
					..default()
				},
				ComboNode::Tree(HashMap::default()),
			),
		)]))
	}

	#[test]
	fn return_skill() {
		let mut combos = Mock_Combos::default();
		combos.expect_peek_next().return_const((
			Skill {
				name: "my skill",
				..default()
			},
			node(),
		));

		assert_eq!(
			Some(Skill {
				name: "my skill",
				..default()
			}),
			combos.peek_next(&SlotKey::Hand(Side::Main), &slots())
		);
	}

	#[test]
	fn return_none() {
		let mut combos = Mock_Combos::default();
		combos.expect_peek_next().return_const(None);

		assert_eq!(
			None as Option<Skill>,
			combos.peek_next(&SlotKey::Hand(Side::Main), &slots())
		);
	}

	#[test]
	fn call_peek_next_with_proper_args() {
		let mut combos = Mock_Combos::default();
		combos
			.expect_peek_next()
			.times(1)
			.with(eq(SlotKey::Hand(Side::Off)), eq(other_slots()))
			.return_const(None);

		let _: Option<Skill> = combos.peek_next(&SlotKey::Hand(Side::Off), &other_slots());
	}
}
