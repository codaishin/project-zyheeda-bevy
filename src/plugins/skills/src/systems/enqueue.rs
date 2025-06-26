use super::get_inputs::Input;
use crate::{
	item::Item,
	skills::Skill,
	traits::{Enqueue, IterWaitingMut, Prime},
};
use bevy::{ecs::component::Mutable, prelude::*};
use common::{
	tools::action_key::slot::SlotKey,
	traits::accessors::get::{GetField, GetRef, Getter},
};

pub(crate) fn enqueue<TSlots, TQueue, TQueuedSkill>(
	input: In<Input>,
	mut agents: Query<(&TSlots, &mut TQueue)>,
	items: Res<Assets<Item>>,
	skills: Res<Assets<Skill>>,
) where
	TSlots: GetRef<SlotKey, Handle<Item>> + Component,
	TQueue:
		Enqueue<(Skill, SlotKey)> + IterWaitingMut<TQueuedSkill> + Component<Mutability = Mutable>,
	TQueuedSkill: Prime + Getter<SlotKey>,
{
	for (slots, mut queue) in &mut agents {
		let queue = queue.as_mut();
		enqueue_new_skills(&input, queue, slots, &items, &skills);
		prime_skills(&input, queue);
	}
}

fn enqueue_new_skills<TSlots, TQueue>(
	input: &In<Input>,
	queue: &mut TQueue,
	slots: &TSlots,
	items: &Assets<Item>,
	skills: &Assets<Skill>,
) where
	TSlots: GetRef<SlotKey, Handle<Item>>,
	TQueue: Enqueue<(Skill, SlotKey)>,
{
	for key in input.just_pressed.iter() {
		enqueue_new_skill(key, queue, slots, items, skills);
	}
}

fn enqueue_new_skill<TSlots, TQueue>(
	key: &SlotKey,
	queue: &mut TQueue,
	slots: &TSlots,
	items: &Assets<Item>,
	skills: &Assets<Skill>,
) where
	TSlots: GetRef<SlotKey, Handle<Item>>,
	TQueue: Enqueue<(Skill, SlotKey)>,
{
	let skill = slots
		.get(key)
		.and_then(|item_handle| items.get(item_handle))
		.and_then(|item| item.skill.as_ref())
		.and_then(|skill_handle| skills.get(skill_handle));

	let Some(skill) = skill else {
		return;
	};

	queue.enqueue((skill.clone(), *key));
}

fn prime_skills<TQueue, TQueuedSkill>(input: &In<Input>, queue: &mut TQueue)
where
	TQueue: IterWaitingMut<TQueuedSkill>,
	TQueuedSkill: Prime + Getter<SlotKey>,
{
	for skill in queue.iter_waiting_mut() {
		if input.pressed.contains(&SlotKey::get_field(skill)) {
			continue;
		}
		skill.prime();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		simple_init,
		test_tools::utils::{SingleThreadedApp, new_handle},
		tools::action_key::slot::Side,
		traits::{handles_localization::Token, mock::Mock, nested_mock::NestedMocks},
	};
	use macros::NestedMocks;
	use mockall::{automock, mock, predicate::eq};
	use std::collections::HashMap;

	#[derive(Resource, Default)]
	struct _Input(Input);

	mock! {
		_SkillQueued {}
		impl Prime for _SkillQueued {
			fn prime(&mut self) {}
		}
		impl Getter<SlotKey> for _SkillQueued {
			fn get(&self) -> SlotKey;
		}
	}

	simple_init!(Mock_SkillQueued);

	#[derive(Component, Default)]
	struct _Skills(HashMap<SlotKey, Handle<Item>>);

	impl GetRef<SlotKey, Handle<Item>> for _Skills {
		fn get<'a>(&'a self, key: &SlotKey) -> Option<&'a Handle<Item>> {
			self.0.get(key)
		}
	}

	#[derive(Component)]
	struct _Enqueue {
		queued: Vec<Mock_SkillQueued>,
	}

	impl Enqueue<(Skill, SlotKey)> for _Enqueue {
		fn enqueue(&mut self, _: (Skill, SlotKey)) {}
	}

	impl IterWaitingMut<Mock_SkillQueued> for _Enqueue {
		fn iter_waiting_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut Mock_SkillQueued>
		where
			Mock_SkillQueued: 'a,
		{
			self.queued.iter_mut()
		}
	}

	struct _SkillLoader;

	fn setup<TEnqueue>(
		items: Vec<(AssetId<Item>, Item)>,
		skills: Vec<(AssetId<Skill>, Skill)>,
	) -> App
	where
		TEnqueue: Enqueue<(Skill, SlotKey)>
			+ IterWaitingMut<Mock_SkillQueued>
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
		app.init_resource::<_Input>();

		app.add_systems(
			Update,
			(move |input: Res<_Input>| input.0.clone())
				.pipe(enqueue::<_Skills, TEnqueue, Mock_SkillQueued>),
		);

		app
	}

	#[allow(static_mut_refs)]
	#[test]
	fn enqueue_skill_from_skills() {
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

		impl IterWaitingMut<Mock_SkillQueued> for _Enqueue {
			fn iter_waiting_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut Mock_SkillQueued>
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
			SlotKey::BottomHand(Side::Right),
			item.clone(),
		)]));
		app.world_mut().spawn((
			skills,
			_Enqueue::new().with_mock(|mock| {
				mock.expect_enqueue()
					.times(1)
					.with(eq((
						Skill {
							token: Token::from("my skill"),
							..default()
						},
						SlotKey::BottomHand(Side::Right),
					)))
					.return_const(());
			}),
		));

		app.world_mut().resource_mut::<_Input>().0.just_pressed =
			vec![SlotKey::BottomHand(Side::Right)];
		app.update();
	}

	#[test]
	fn prime_skill_if_its_slot_is_not_pressed() {
		let right = SlotKey::BottomHand(Side::Right);
		let left = SlotKey::BottomHand(Side::Left);
		let mut app = setup::<_Enqueue>(vec![], vec![]);
		app.world_mut().spawn((
			_Skills::default(),
			_Enqueue {
				queued: vec![Mock_SkillQueued::new_mock(|mock| {
					mock.expect_get().return_const(left);
					mock.expect_prime().return_const(());
				})],
			},
		));
		app.world_mut().resource_mut::<_Input>().0.pressed = vec![right];

		app.update();
	}

	#[test]
	fn prime_all_in_queue() {
		let mut app = setup::<_Enqueue>(vec![], vec![]);
		app.world_mut().spawn((
			_Skills::default(),
			_Enqueue {
				queued: vec![
					Mock_SkillQueued::new_mock(|mock| {
						mock.expect_prime().times(1).return_const(());
						mock.expect_get()
							.return_const(SlotKey::BottomHand(Side::Right));
					}),
					Mock_SkillQueued::new_mock(|mock| {
						mock.expect_prime().times(1).return_const(());
						mock.expect_get()
							.return_const(SlotKey::BottomHand(Side::Left));
					}),
				],
			},
		));

		app.update();
	}
}
