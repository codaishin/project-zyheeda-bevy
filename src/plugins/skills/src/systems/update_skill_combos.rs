use crate::{
	components::slots::Slots,
	item::SkillItem,
	skills::QueuedSkill,
	traits::{AdvanceCombo2, IterAddedMut},
};
use bevy::prelude::*;
use common::traits::accessors::get::GetRef;

pub(crate) fn update_skill_combos<TCombos, TQueue>(
	mut agents: Query<(&mut TCombos, &mut TQueue, &Slots)>,
) where
	TCombos: AdvanceCombo2 + Component,
	TQueue: IterAddedMut<QueuedSkill> + Component,
{
	for (mut combos, mut queue, slots) in &mut agents {
		if queue.added_none() {
			continue;
		}
		for skill in queue.iter_added_mut() {
			update_skill_with_advanced_combo(&mut combos, skill, slots);
		}
	}
}

fn update_skill_with_advanced_combo<TCombos>(
	combos: &mut Mut<TCombos>,
	added: &mut QueuedSkill,
	slots: &Slots,
) where
	TCombos: AdvanceCombo2,
{
	let QueuedSkill {
		skill, slot_key, ..
	} = added;
	let Some(item): Option<&SkillItem> = slots.get(slot_key) else {
		return;
	};
	let Some(advanced) = combos.advance2(&added.slot_key, &item.content.item_type) else {
		return;
	};

	*skill = advanced;
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::slots::Slots,
		item::{item_type::SkillItemType, SkillItem, SkillItemContent},
		skills::Skill,
		slot_key::SlotKey,
	};
	use bevy::ecs::system::RunSystemOnce;
	use common::{components::Side, test_tools::utils::Changed, traits::nested_mock::NestedMocks};
	use macros::NestedMocks;
	use mockall::{mock, predicate::eq};

	#[derive(Component, NestedMocks)]
	struct _Combos {
		mock: Mock_Combos,
	}

	mock! {
		_Combos {}
		impl AdvanceCombo2 for _Combos {
			fn advance2(&mut self, trigger: &SlotKey, item_type: &SkillItemType) -> Option<Skill> {}
		}
	}

	impl AdvanceCombo2 for _Combos {
		fn advance2(&mut self, trigger: &SlotKey, item_type: &SkillItemType) -> Option<Skill> {
			self.mock.advance2(trigger, item_type)
		}
	}

	#[derive(Component, Default, PartialEq, Debug)]
	struct _Queue {
		added: Vec<QueuedSkill>,
	}

	impl IterAddedMut<QueuedSkill> for _Queue {
		fn added_none(&self) -> bool {
			self.added.is_empty()
		}

		fn iter_added_mut<'a>(&'a mut self) -> impl DoubleEndedIterator<Item = &'a mut QueuedSkill>
		where
			QueuedSkill: 'a,
		{
			self.added.iter_mut()
		}
	}

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn call_advance_with_matching_slot_key_and_item_type() {
		let mut app = setup();
		app.world_mut().spawn((
			_Combos::new().with_mock(|mock| {
				mock.expect_advance2()
					.times(1)
					.with(
						eq(SlotKey::BottomHand(Side::Right)),
						eq(SkillItemType::ForceEssence),
					)
					.return_const(Skill::default());
				mock.expect_advance2()
					.times(1)
					.with(
						eq(SlotKey::BottomHand(Side::Left)),
						eq(SkillItemType::Pistol),
					)
					.return_const(Skill::default());
			}),
			_Queue {
				added: vec![
					QueuedSkill {
						slot_key: SlotKey::BottomHand(Side::Right),
						..default()
					},
					QueuedSkill {
						slot_key: SlotKey::BottomHand(Side::Left),
						..default()
					},
				],
			},
			Slots::new([
				(
					SlotKey::BottomHand(Side::Right),
					Some(SkillItem {
						content: SkillItemContent {
							item_type: SkillItemType::ForceEssence,
							..default()
						},
						..default()
					}),
				),
				(
					SlotKey::BottomHand(Side::Left),
					Some(SkillItem {
						content: SkillItemContent {
							item_type: SkillItemType::Pistol,
							..default()
						},
						..default()
					}),
				),
			]),
		));

		app.world_mut()
			.run_system_once(update_skill_combos::<_Combos, _Queue>);
	}

	#[test]
	fn update_skill_with_combo_skills() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				_Combos::new().with_mock(|mock| {
					mock.expect_advance2().return_const(Skill {
						name: "replace a".to_owned(),
						..default()
					});
				}),
				_Queue {
					added: vec![QueuedSkill {
						skill: Skill {
							name: "skill a".to_owned(),
							..default()
						},
						..default()
					}],
				},
				Slots::new([(SlotKey::default(), Some(SkillItem::default()))]),
			))
			.id();

		app.world_mut()
			.run_system_once(update_skill_combos::<_Combos, _Queue>);

		let agent = app.world().entity(agent);

		assert_eq!(
			Some(&_Queue {
				added: vec![QueuedSkill {
					skill: Skill {
						name: "replace a".to_owned(),
						..default()
					},
					slot_key: SlotKey::BottomHand(Side::Right),
					..default()
				}],
			}),
			agent.get::<_Queue>()
		);
	}

	#[test]
	fn queue_not_marked_changed_when_non_added() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Changed::<_Queue>::new(false),
				_Combos::new(),
				_Queue::default(),
				Slots::default(),
			))
			.id();

		app.add_systems(Update, update_skill_combos::<_Combos, _Queue>);
		app.add_systems(PostUpdate, Changed::<_Queue>::detect);
		app.update(); // changed always true, because target was just added
		app.update();

		assert_eq!(
			Some(&Changed::new(false)),
			app.world().entity(entity).get::<Changed<_Queue>>(),
		)
	}
}
