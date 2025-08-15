use crate::{
	item::Item,
	skills::Skill,
	traits::{Enqueue, IterHoldingMut, ReleaseSkill},
};
use bevy::{ecs::component::Mutable, prelude::*};
use common::{
	tools::action_key::slot::SlotKey,
	traits::{
		accessors::get::{GetRef, RefAs, RefInto},
		handles_skill_behaviors::HoldSkills,
	},
};

impl<T> EnqueueSystem for T where T: Component + HoldSkills {}

pub(crate) trait EnqueueSystem: Component + HoldSkills + Sized {
	fn enqueue<TSlots, TQueue>(
		mut agents: Query<(&TSlots, &mut TQueue, &Self)>,
		items: Res<Assets<Item>>,
		skills: Res<Assets<Skill>>,
	) where
		TQueue: Enqueue<(Skill, SlotKey)> + IterHoldingMut + Component<Mutability = Mutable>,
		TQueue::TItem: ReleaseSkill + for<'a> RefInto<'a, SlotKey>,
		for<'a> TSlots: GetRef<SlotKey, TValue<'a> = &'a Handle<Item>> + Component,
	{
		for (slots, mut queue, agent) in &mut agents {
			for key in agent.started_holding() {
				let Some(skill) = get_skill(&key, slots, &items, &skills) else {
					continue;
				};
				queue.enqueue((skill.clone(), key));
			}

			let holding = agent.holding().collect::<Vec<_>>();
			for skill in queue.iter_holding_mut() {
				let key = (*skill).ref_as::<SlotKey>();
				if holding.contains(&key) {
					continue;
				}
				skill.release_skill();
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
	use common::{tools::action_key::slot::PlayerSlot, traits::handles_localization::Token};
	use macros::NestedMocks;
	use mockall::{automock, mock, predicate::eq};
	use std::{collections::HashMap, iter::Cloned, slice::Iter};
	use testing::{Mock, NestedMocks, SingleThreadedApp, new_handle, simple_init};

	mock! {
		_SkillQueued {}
		impl ReleaseSkill for _SkillQueued {
			fn release_skill(&mut self) {}
		}
		impl RefInto<'_, SlotKey> for _SkillQueued {
			fn ref_into(&self) -> SlotKey {}
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

	#[derive(Component)]
	struct _Agent {
		started_holding: Vec<SlotKey>,
		holding: Vec<SlotKey>,
	}

	impl HoldSkills for _Agent {
		type Iter<'a> = Cloned<Iter<'a, SlotKey>>;

		fn holding(&self) -> Self::Iter<'_> {
			self.holding.iter().cloned()
		}

		fn started_holding(&self) -> Self::Iter<'_> {
			self.started_holding.iter().cloned()
		}
	}

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
			item_assets.insert(id, item);
		}

		for (id, skill) in skills {
			skill_assets.insert(id, skill);
		}

		app.insert_resource(item_assets);
		app.insert_resource(skill_assets);
		app.add_systems(Update, _Agent::enqueue::<_Skills, TEnqueue>);

		app
	}

	#[test]
	fn enqueue_skill_in_started_holding() {
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
			SlotKey::from(PlayerSlot::UPPER_R),
			item.clone(),
		)]));
		app.world_mut().spawn((
			skills,
			_Agent {
				started_holding: vec![SlotKey::from(PlayerSlot::UPPER_R)],
				holding: vec![],
			},
			_Enqueue::new().with_mock(|mock| {
				mock.expect_enqueue()
					.times(1)
					.with(eq((
						Skill {
							token: Token::from("my skill"),
							..default()
						},
						SlotKey::from(PlayerSlot::UPPER_R),
					)))
					.return_const(());
			}),
		));

		app.update();
	}

	#[test]
	fn release_skill_release_when_not_in_holding() {
		let mut app = setup::<_Enqueue>(vec![], vec![]);
		app.world_mut().spawn((
			_Skills::default(),
			_Agent {
				started_holding: vec![],
				holding: vec![SlotKey::from(PlayerSlot::LOWER_R)],
			},
			_Enqueue {
				queued: HashMap::from([
					(
						SlotKey::from(PlayerSlot::LOWER_L),
						Mock_SkillQueued::new_mock(|mock| {
							mock.expect_release_skill().times(1).return_const(());
							mock.expect_ref_into()
								.return_const(SlotKey::from(PlayerSlot::LOWER_L));
						}),
					),
					(
						SlotKey::from(PlayerSlot::LOWER_R),
						Mock_SkillQueued::new_mock(|mock| {
							mock.expect_release_skill().never().return_const(());
							mock.expect_ref_into()
								.return_const(SlotKey::from(PlayerSlot::LOWER_R));
						}),
					),
				]),
			},
		));

		app.update();
	}
}
