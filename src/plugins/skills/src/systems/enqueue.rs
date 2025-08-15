use crate::{
	events::skill::SkillEvent,
	item::Item,
	skills::Skill,
	traits::{Enqueue, GetHoldingMut, ReleaseSkill},
};
use bevy::{ecs::component::Mutable, prelude::*};
use common::{tools::action_key::slot::SlotKey, traits::accessors::get::GetRef};

pub(crate) fn enqueue<TSlots, TQueue>(
	mut skill_events: EventReader<SkillEvent>,
	mut agents: Query<(&TSlots, &mut TQueue)>,
	items: Res<Assets<Item>>,
	skills: Res<Assets<Skill>>,
) where
	TQueue: Enqueue<(Skill, SlotKey)> + GetHoldingMut<SlotKey> + Component<Mutability = Mutable>,
	TQueue::TItem: ReleaseSkill,
	for<'a> TSlots: GetRef<SlotKey, TValue<'a> = &'a Handle<Item>> + Component,
{
	for event in skill_events.read() {
		match event {
			SkillEvent::Hold { agent, key } => {
				let Ok((slots, mut queue)) = agents.get_mut(*agent) else {
					continue;
				};

				if let Some(skill) = get_skill(key, slots, &items, &skills) {
					queue.enqueue((skill.clone(), *key));
				};
			}
			SkillEvent::Release { agent, key } => {
				let Ok((_, mut queue)) = agents.get_mut(*agent) else {
					continue;
				};

				for skill in queue.get_holding_mut(*key) {
					skill.release_skill();
				}
			}
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
	use common::{
		tools::action_key::slot::{PlayerSlot, Side},
		traits::handles_localization::Token,
	};
	use macros::NestedMocks;
	use mockall::{automock, mock, predicate::eq};
	use std::collections::HashMap;
	use testing::{Mock, NestedMocks, SingleThreadedApp, new_handle, simple_init};

	mock! {
		_SkillQueued {}
		impl ReleaseSkill for _SkillQueued {
			fn release_skill(&mut self) {}
		}
	}

	simple_init!(Mock_SkillQueued);

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

	impl GetHoldingMut<SlotKey> for _Enqueue {
		type TItem = Mock_SkillQueued;

		fn get_holding_mut<'a>(
			&'a mut self,
			key: SlotKey,
		) -> impl Iterator<Item = &'a mut Mock_SkillQueued>
		where
			Mock_SkillQueued: 'a,
		{
			self.queued
				.iter_mut()
				.filter(move |(k, _)| k == &&key)
				.map(|(_, item)| item)
		}
	}

	struct _SkillLoader;

	fn setup<TEnqueue>(
		items: Vec<(AssetId<Item>, Item)>,
		skills: Vec<(AssetId<Skill>, Skill)>,
	) -> App
	where
		TEnqueue: Enqueue<(Skill, SlotKey)>
			+ GetHoldingMut<SlotKey, TItem = Mock_SkillQueued>
			+ Component<Mutability = Mutable>,
	{
		let mut app = App::new().single_threaded(Update);
		let mut item_assets = Assets::<Item>::default();
		let mut skill_assets = Assets::<Skill>::default();

		for (id, item) in items {
			item_assets.insert(id, item);
		}

		for (id, skill) in skills {
			skill_assets.insert(id, skill);
		}

		app.insert_resource(item_assets);
		app.insert_resource(skill_assets);
		app.add_event::<SkillEvent>();
		app.add_systems(Update, enqueue::<_Skills, TEnqueue>);

		app
	}

	#[allow(static_mut_refs)]
	#[test]
	fn enqueue_skill_hold() {
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

		static mut EMPTY: [Mock_SkillQueued; 0] = [];

		impl GetHoldingMut<SlotKey> for _Enqueue {
			type TItem = Mock_SkillQueued;

			fn get_holding_mut<'a>(
				&mut self,
				_: SlotKey,
			) -> impl Iterator<Item = &'a mut Mock_SkillQueued>
			where
				Mock_SkillQueued: 'a,
			{
				unsafe { EMPTY.iter_mut() }
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
			SlotKey::from(PlayerSlot::Lower(Side::Right)),
			item.clone(),
		)]));
		let agent = app
			.world_mut()
			.spawn((
				skills,
				_Enqueue::new().with_mock(|mock| {
					mock.expect_enqueue()
						.times(1)
						.with(eq((
							Skill {
								token: Token::from("my skill"),
								..default()
							},
							SlotKey::from(PlayerSlot::Lower(Side::Right)),
						)))
						.return_const(());
				}),
			))
			.id();
		app.world_mut().send_event(SkillEvent::Hold {
			agent,
			key: SlotKey::from(PlayerSlot::Lower(Side::Right)),
		});

		app.update();
	}

	#[test]
	fn release_skill_release() {
		let mut app = setup::<_Enqueue>(vec![], vec![]);
		let agent = app
			.world_mut()
			.spawn((
				_Skills::default(),
				_Enqueue {
					queued: HashMap::from([(
						SlotKey::from(PlayerSlot::Lower(Side::Left)),
						Mock_SkillQueued::new_mock(|mock| {
							mock.expect_release_skill().times(1).return_const(());
						}),
					)]),
				},
			))
			.id();
		app.world_mut().send_event(SkillEvent::Release {
			agent,
			key: SlotKey::from(PlayerSlot::Lower(Side::Left)),
		});

		app.update();
	}
}
