use crate::{
	components::slots::Slots,
	item::Item,
	skills::QueuedSkill,
	traits::{AdvanceCombo, IterAddedMut},
};
use bevy::{ecs::component::Mutable, prelude::*};
use common::traits::accessors::get::GetRef;

impl<T> ComboQueueUpdate for T where T: Component<Mutability = Mutable> + AdvanceCombo {}

pub(crate) trait ComboQueueUpdate:
	Component<Mutability = Mutable> + AdvanceCombo + Sized
{
	fn update<TQueue>(mut agents: Query<(&mut Self, &mut TQueue, &Slots)>, items: Res<Assets<Item>>)
	where
		TQueue: IterAddedMut<TItem = QueuedSkill> + Component<Mutability = Mutable>,
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
}

fn update_skill_with_advanced_combo<TCombos>(
	combos: &mut Mut<TCombos>,
	added: &mut QueuedSkill,
	slots: &Slots,
	items: &Res<Assets<Item>>,
) where
	TCombos: AdvanceCombo,
{
	let QueuedSkill { skill, key, .. } = added;
	let Some(item_handle) = slots.get(key) else {
		return;
	};
	let Some(item) = items.get(item_handle.id()) else {
		return;
	};
	let Some(advanced) = combos.advance_combo(*key, &item.item_type) else {
		return;
	};

	*skill = advanced;
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{components::slots::Slots, item::Item, skills::Skill};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{
		tools::{
			action_key::slot::{PlayerSlot, Side, SlotKey},
			item_type::ItemType,
		},
		traits::handles_localization::Token,
	};
	use macros::NestedMocks;
	use mockall::{mock, predicate::eq};
	use std::collections::HashMap;
	use testing::{IsChanged, NestedMocks};

	#[derive(Component, NestedMocks)]
	struct _Combos {
		mock: Mock_Combos,
	}

	mock! {
		_Combos {}
		impl AdvanceCombo for _Combos {
			fn advance_combo(&mut self, trigger: SlotKey, item_type: &ItemType) -> Option<Skill> {}
		}
	}

	impl AdvanceCombo for _Combos {
		fn advance_combo(&mut self, trigger: SlotKey, item_type: &ItemType) -> Option<Skill> {
			self.mock.advance_combo(trigger, item_type)
		}
	}

	#[derive(Component, Default, PartialEq, Debug)]
	struct _Queue {
		added: Vec<QueuedSkill>,
	}

	impl IterAddedMut for _Queue {
		type TItem = QueuedSkill;

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
				SlotKey::from(PlayerSlot::Lower(Side::Right)),
				Some(Item {
					item_type: ItemType::ForceEssence,
					..default()
				}),
			),
			(
				SlotKey::from(PlayerSlot::Lower(Side::Left)),
				Some(Item {
					item_type: ItemType::Pistol,
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
						eq(SlotKey::from(PlayerSlot::Lower(Side::Right))),
						eq(ItemType::ForceEssence),
					)
					.return_const(Skill::default());
				mock.expect_advance_combo()
					.times(1)
					.with(
						eq(SlotKey::from(PlayerSlot::Lower(Side::Left))),
						eq(ItemType::Pistol),
					)
					.return_const(Skill::default());
			}),
			_Queue {
				added: vec![
					QueuedSkill {
						key: SlotKey::from(PlayerSlot::Lower(Side::Right)),
						..default()
					},
					QueuedSkill {
						key: SlotKey::from(PlayerSlot::Lower(Side::Left)),
						..default()
					},
				],
			},
			slots,
		));

		app.world_mut().run_system_once(_Combos::update::<_Queue>)
	}

	#[test]
	fn update_skill_with_combo_skills() -> Result<(), RunSystemError> {
		let (slots, items) = setup_slots([(SlotKey(0), Some(Item::default()))]);
		let mut app = setup_app(items);
		let agent = app
			.world_mut()
			.spawn((
				_Combos::new().with_mock(|mock| {
					mock.expect_advance_combo().return_const(Skill {
						token: Token::from("replace a"),
						..default()
					});
				}),
				_Queue {
					added: vec![QueuedSkill {
						skill: Skill {
							token: Token::from("skill a"),
							..default()
						},
						..default()
					}],
				},
				slots,
			))
			.id();

		app.world_mut().run_system_once(_Combos::update::<_Queue>)?;

		let agent = app.world().entity(agent);

		assert_eq!(
			Some(&_Queue {
				added: vec![QueuedSkill {
					skill: Skill {
						token: Token::from("replace a"),
						..default()
					},
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
			.spawn((_Combos::new(), _Queue::default(), slots))
			.id();

		app.add_systems(Update, _Combos::update::<_Queue>);
		app.add_systems(PostUpdate, IsChanged::<_Queue>::detect);
		app.update(); // changed always true, because target was just added
		app.update();

		assert_eq!(
			Some(&IsChanged::<_Queue>::FALSE),
			app.world().entity(entity).get::<IsChanged<_Queue>>(),
		);
	}
}
