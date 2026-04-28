use crate::{
	components::held_slots::HeldSlots,
	item::Item,
	skills::Skill,
	traits::{Enqueue, IterHoldingMut, ReleaseSkill},
};
use bevy::{ecs::component::Mutable, prelude::*};
use common::{
	tools::action_key::slot::SlotKey,
	traits::accessors::get::{GetRef, View},
};

impl<T> EnqueueSystem for T where
	T: Component<Mutability = Mutable>
		+ Enqueue<(Skill, SlotKey)>
		+ IterHoldingMut<TItem: ReleaseSkill + View<SlotKey>>
{
}

pub(crate) trait EnqueueSystem:
	Component<Mutability = Mutable>
	+ Enqueue<(Skill, SlotKey)>
	+ IterHoldingMut<TItem: ReleaseSkill + View<SlotKey>>
	+ Sized
{
	fn enqueue_system<TSlots>(
		agents: Query<(&mut Self, &TSlots, &mut HeldSlots)>,
		items: Res<Assets<Item>>,
		skills: Res<Assets<Skill>>,
	) where
		for<'a> TSlots: GetRef<SlotKey, TValue<'a> = &'a Handle<Item>> + Component,
	{
		for (mut queue, slots, mut held_slots) in agents {
			for key in held_slots.iter_new() {
				let Some(skill) = get_skill(key, slots, &items, &skills) else {
					continue;
				};
				queue.enqueue((skill.clone(), *key));
			}

			for skill in queue.iter_holding_mut() {
				let key = skill.view();
				if held_slots.contains(&key) {
					continue;
				}
				skill.release_skill();
			}

			held_slots.rotate();
		}
	}
}

fn get_skill<'a, TSlots>(
	key: &SlotKey,
	slots: &'a TSlots,
	items: &'a Assets<Item>,
	skills: &'a Assets<Skill>,
) -> Option<&'a Skill>
where
	TSlots: GetRef<SlotKey, TValue<'a> = &'a Handle<Item>>,
{
	slots
		.get_ref(key)
		.and_then(|item_handle| items.get(item_handle))
		.and_then(|item| item.skill.as_ref())
		.and_then(|skill_handle| skills.get(skill_handle))
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{tools::action_key::slot::HandSlot, traits::handles_localization::Token};
	use macros::{NestedMocks, simple_mock};
	use mockall::{automock, predicate::eq};
	use std::collections::HashMap;
	use testing::{Mock, NestedMocks, SingleThreadedApp, new_handle};

	simple_mock! {
		_SkillQueued {}
		impl ReleaseSkill for _SkillQueued {
			fn release_skill(&mut self) {}
		}
		impl View< SlotKey> for _SkillQueued {
			fn view(&self) -> SlotKey {}
		}
	}

	#[derive(Component, Default)]
	struct _Skills(HashMap<SlotKey, Handle<Item>>);

	impl GetRef<SlotKey> for _Skills {
		type TValue<'a>
			= &'a Handle<Item>
		where
			Self: 'a;

		fn get_ref<'a>(&'a self, key: &SlotKey) -> Option<&'a Handle<Item>> {
			self.0.get(key)
		}
	}

	#[derive(Component)]
	struct _Enqueue {
		queued: HashMap<SlotKey, Mock_SkillQueued>,
	}

	impl Enqueue<(Skill, SlotKey)> for _Enqueue {
		fn enqueue(&mut self, _: (Skill, SlotKey)) {}
	}

	impl IterHoldingMut for _Enqueue {
		type TItem = Mock_SkillQueued;

		fn iter_holding_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut Mock_SkillQueued>
		where
			Mock_SkillQueued: 'a,
		{
			self.queued.iter_mut().map(|(_, item)| item)
		}
	}

	struct _SkillLoader;

	fn setup<TEnqueue>(
		items: Vec<(AssetId<Item>, Item)>,
		skills: Vec<(AssetId<Skill>, Skill)>,
	) -> App
	where
		TEnqueue: Enqueue<(Skill, SlotKey)>
			+ IterHoldingMut<TItem = Mock_SkillQueued>
			+ Component<Mutability = Mutable>,
	{
		let mut app = App::new().single_threaded(Update);
		let mut item_assets = Assets::<Item>::default();
		let mut skill_assets = Assets::<Skill>::default();

		for (id, item) in items {
			_ = item_assets.insert(id, item);
		}

		for (id, skill) in skills {
			_ = skill_assets.insert(id, skill);
		}

		app.insert_resource(item_assets);
		app.insert_resource(skill_assets);
		app.add_systems(Update, TEnqueue::enqueue_system::<_Skills>);

		app
	}

	#[test]
	fn enqueue_skill_in_current_held_slots_but_not_in_old_held_slots() {
		#[derive(Component, NestedMocks)]
		struct _Enqueue {
			mock: Mock_Enqueue,
		}

		#[automock]
		impl Enqueue<(Skill, SlotKey)> for _Enqueue {
			fn enqueue(&mut self, item: (Skill, SlotKey)) {
				self.mock.enqueue(item)
			}
		}

		impl IterHoldingMut for _Enqueue {
			type TItem = Mock_SkillQueued;

			fn iter_holding_mut<'a>(&mut self) -> impl Iterator<Item = &'a mut Mock_SkillQueued>
			where
				Mock_SkillQueued: 'a,
			{
				std::iter::empty()
			}
		}

		let item = new_handle();
		let skill = new_handle();
		let mut app = setup::<_Enqueue>(
			vec![(
				item.id(),
				Item {
					skill: Some(skill.clone()),
					..default()
				},
			)],
			vec![(
				skill.id(),
				Skill {
					token: Token::from("my skill"),
					..default()
				},
			)],
		);

		let skills = _Skills(HashMap::from([(
			SlotKey::from(HandSlot::Right),
			item.clone(),
		)]));
		let mut held_slots = HeldSlots::from([SlotKey::from(HandSlot::Left)]);
		held_slots.rotate();
		held_slots.extend([
			SlotKey::from(HandSlot::Left),
			SlotKey::from(HandSlot::Right),
		]);
		app.world_mut().spawn((
			skills,
			held_slots,
			_Enqueue::new().with_mock(|mock| {
				mock.expect_enqueue()
					.times(1)
					.with(eq((
						Skill {
							token: Token::from("my skill"),
							..default()
						},
						SlotKey::from(HandSlot::Right),
					)))
					.return_const(());
			}),
		));

		app.update();
	}

	#[test]
	fn release_skill_when_not_in_current_held_slots() {
		let mut app = setup::<_Enqueue>(vec![], vec![]);
		app.world_mut().spawn((
			_Skills::default(),
			HeldSlots::from([SlotKey::from(HandSlot::Left)]),
			_Enqueue {
				queued: HashMap::from([
					(
						SlotKey::from(HandSlot::Right),
						Mock_SkillQueued::new_mock(|mock| {
							mock.expect_release_skill().times(1).return_const(());
							mock.expect_view()
								.return_const(SlotKey::from(HandSlot::Right));
						}),
					),
					(
						SlotKey::from(HandSlot::Left),
						Mock_SkillQueued::new_mock(|mock| {
							mock.expect_release_skill().never().return_const(());
							mock.expect_view()
								.return_const(SlotKey::from(HandSlot::Left));
						}),
					),
				]),
			},
		));

		app.update();
	}

	#[test]
	fn rotate_held_slots() {
		let mut app = setup::<_Enqueue>(vec![], vec![]);
		let entity = app
			.world_mut()
			.spawn((
				_Skills::default(),
				HeldSlots::from([SlotKey::from(HandSlot::Left)]),
				_Enqueue {
					queued: HashMap::from([]),
				},
			))
			.id();

		app.update();

		let mut expected = HeldSlots::from([SlotKey::from(HandSlot::Left)]);
		expected.rotate();
		assert_eq!(
			Some(&expected),
			app.world().entity(entity).get::<HeldSlots>(),
		);
	}
}
