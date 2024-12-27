use crate::{
	components::slots::Slots,
	item::Item,
	skills::QueuedSkill,
	traits::{AdvanceCombo, IterAddedMut},
};
use bevy::prelude::*;
use common::traits::accessors::get::GetRef;

pub(crate) fn update_skill_combos<TCombos, TQueue>(
	mut agents: Query<(&mut TCombos, &mut TQueue, &Slots)>,
	items: Res<Assets<Item>>,
) where
	TCombos: AdvanceCombo + Component,
	TQueue: IterAddedMut<QueuedSkill> + Component,
{
	for (mut combos, mut queue, slots) in &mut agents {
		if queue.added_none() {
			continue;
		}
		for skill in queue.iter_added_mut() {
			update_skill_with_advanced_combo(&mut combos, skill, slots, &items);
		}
	}
}

fn update_skill_with_advanced_combo<TCombos>(
	combos: &mut Mut<TCombos>,
	added: &mut QueuedSkill,
	slots: &Slots,
	items: &Res<Assets<Item>>,
) where
	TCombos: AdvanceCombo,
{
	let QueuedSkill {
		skill, slot_key, ..
	} = added;
	let Some(item_handle) = slots.get(slot_key) else {
		return;
	};
	let Some(item) = items.get(item_handle.id()) else {
		return;
	};
	let Some(advanced) = combos.advance_combo(&added.slot_key, &item.item_type) else {
		return;
	};

	*skill = advanced;
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::slots::Slots,
		item::{item_type::SkillItemType, Item},
		skills::Skill,
	};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{
		test_tools::utils::Changed,
		tools::slot_key::{Side, SlotKey},
		traits::nested_mock::NestedMocks,
	};
	use macros::NestedMocks;
	use mockall::{mock, predicate::eq};
	use std::collections::HashMap;

	#[derive(Component, NestedMocks)]
	struct _Combos {
		mock: Mock_Combos,
	}

	mock! {
		_Combos {}
		impl AdvanceCombo for _Combos {
			fn advance_combo(&mut self, trigger: &SlotKey, item_type: &SkillItemType) -> Option<Skill> {}
		}
	}

	impl AdvanceCombo for _Combos {
		fn advance_combo(&mut self, trigger: &SlotKey, item_type: &SkillItemType) -> Option<Skill> {
			self.mock.advance_combo(trigger, item_type)
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

	fn setup_app(items: Assets<Item>) -> App {
		let mut app = App::new();
		app.insert_resource(items);

		app
	}

	fn setup_slots<const N: usize>(
		slots_and_items: [(SlotKey, Option<Item>); N],
	) -> (Slots, Assets<Item>) {
		let mut assets = Assets::default();
		let mut slots = HashMap::default();

		for (slot_key, item) in slots_and_items {
			let Some(item) = item else {
				continue;
			};
			let handle = assets.add(item);
			slots.insert(slot_key, Some(handle));
		}

		(Slots(slots), assets)
	}

	#[test]
	fn call_advance_with_matching_slot_key_and_item_type() -> Result<(), RunSystemError> {
		let (slots, items) = setup_slots([
			(
				SlotKey::BottomHand(Side::Right),
				Some(Item {
					item_type: SkillItemType::ForceEssence,
					..default()
				}),
			),
			(
				SlotKey::BottomHand(Side::Left),
				Some(Item {
					item_type: SkillItemType::Pistol,
					..default()
				}),
			),
		]);
		let mut app = setup_app(items);
		app.world_mut().spawn((
			_Combos::new().with_mock(|mock| {
				mock.expect_advance_combo()
					.times(1)
					.with(
						eq(SlotKey::BottomHand(Side::Right)),
						eq(SkillItemType::ForceEssence),
					)
					.return_const(Skill::default());
				mock.expect_advance_combo()
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
			slots,
		));

		app.world_mut()
			.run_system_once(update_skill_combos::<_Combos, _Queue>)
	}

	#[test]
	fn update_skill_with_combo_skills() -> Result<(), RunSystemError> {
		let (slots, items) = setup_slots([(SlotKey::default(), Some(Item::default()))]);
		let mut app = setup_app(items);
		let agent = app
			.world_mut()
			.spawn((
				_Combos::new().with_mock(|mock| {
					mock.expect_advance_combo().return_const(Skill {
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
				slots,
			))
			.id();

		app.world_mut()
			.run_system_once(update_skill_combos::<_Combos, _Queue>)?;

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
		Ok(())
	}

	#[test]
	fn queue_not_marked_changed_when_non_added() {
		let (slots, items) = setup_slots([]);
		let mut app = setup_app(items);
		let entity = app
			.world_mut()
			.spawn((
				Changed::<_Queue>::new(false),
				_Combos::new(),
				_Queue::default(),
				slots,
			))
			.id();

		app.add_systems(Update, update_skill_combos::<_Combos, _Queue>);
		app.add_systems(PostUpdate, Changed::<_Queue>::detect);
		app.update(); // changed always true, because target was just added
		app.update();

		assert_eq!(
			Some(&Changed::new(false)),
			app.world().entity(entity).get::<Changed<_Queue>>(),
		);
	}
}
