use super::{AdvanceCombo2, PeekNext2, SetNextCombo};
use crate::{
	components::combo_node::ComboNode,
	item::item_type::SkillItemType,
	skills::Skill,
	slot_key::SlotKey,
};

impl<T> AdvanceCombo2 for T
where
	T: PeekNext2<(Skill, ComboNode)> + SetNextCombo<Option<ComboNode>>,
{
	fn advance2(&mut self, trigger: &SlotKey, item_type: &SkillItemType) -> Option<Skill> {
		let Some((skill, next_combo)) = self.peek_next2(trigger, item_type) else {
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
	use bevy::utils::default;
	use common::{components::Side, simple_init, traits::mock::Mock};
	use mockall::{mock, predicate::eq};

	mock! {
		_Combos {}
		impl PeekNext2<(Skill, ComboNode)> for _Combos {
			fn peek_next2(&self, trigger: &SlotKey, slots: &SkillItemType) -> Option<(Skill, ComboNode)>;
		}
		impl SetNextCombo<Option<ComboNode>> for _Combos {
			fn set_next_combo(&mut self, value: Option<ComboNode>);
		}
	}

	simple_init!(Mock_Combos);

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
	fn call_set_next_combo_with_next_when_peek_was_some() {
		let mut combos = Mock_Combos::new_mock(|mock| {
			mock.expect_peek_next2()
				.with(
					eq(SlotKey::BottomHand(Side::Right)),
					eq(SkillItemType::Pistol),
				)
				.return_const((Skill::default(), node()));
			mock.expect_set_next_combo()
				.times(1)
				.with(eq(Some(node())))
				.return_const(());
		});

		combos.advance2(&SlotKey::default(), &SkillItemType::default());
	}

	#[test]
	fn return_skill_when_peek_next_was_some() {
		let mut combos = Mock_Combos::new_mock(|mock| {
			mock.expect_peek_next2().return_const((
				Skill {
					name: "return this".to_owned(),
					..default()
				},
				node(),
			));
			mock.expect_set_next_combo().return_const(());
		});

		let skill = combos.advance2(&SlotKey::default(), &SkillItemType::default());

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
		let mut combos = Mock_Combos::new_mock(|mock| {
			mock.expect_peek_next2().return_const(None);
			mock.expect_set_next_combo()
				.times(1)
				.with(eq(None))
				.return_const(());
		});

		combos.advance2(&SlotKey::default(), &SkillItemType::default());
	}

	#[test]
	fn return_none_when_peek_next_was_none() {
		let mut combos = Mock_Combos::new_mock(|mock| {
			mock.expect_peek_next2().return_const(None);
			mock.expect_set_next_combo().return_const(());
		});

		let skill = combos.advance2(&SlotKey::default(), &SkillItemType::default());

		assert_eq!(None, skill);
	}

	#[test]
	fn call_peek_next_with_correct_args() {
		let mut combos = Mock_Combos::new_mock(|mock| {
			mock.expect_peek_next2()
				.times(1)
				.with(
					eq(SlotKey::BottomHand(Side::Left)),
					eq(SkillItemType::ForceEssence),
				)
				.return_const(None);
			mock.expect_set_next_combo().return_const(());
		});

		combos.advance2(
			&SlotKey::BottomHand(Side::Left),
			&SkillItemType::ForceEssence,
		);
	}
}
