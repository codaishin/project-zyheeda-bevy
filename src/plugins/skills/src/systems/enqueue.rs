use super::get_inputs::Input;
use crate::{
	skills::Skill,
	slot_key::SlotKey,
	traits::{Enqueue, IterMut, Matches, Prime},
};
use bevy::ecs::{
	component::Component,
	system::{In, Query},
};
use common::traits::accessors::get::GetRef;

pub(crate) fn enqueue<
	TSlots: GetRef<SlotKey, Skill> + Component,
	TQueue: Enqueue<(Skill, SlotKey)> + IterMut<TQueuedSkill> + Component,
	TQueuedSkill: Prime + Matches<SlotKey>,
>(
	input: In<Input>,
	mut agents: Query<(&TSlots, &mut TQueue)>,
) {
	for (slots, mut queue) in &mut agents {
		let queue = queue.as_mut();
		enqueue_new_skills(&input, queue, slots);
		prime_skills(&input, queue);
	}
}

fn enqueue_new_skills<TSlots: GetRef<SlotKey, Skill>, TQueue: Enqueue<(Skill, SlotKey)>>(
	input: &In<Input>,
	queue: &mut TQueue,
	slots: &TSlots,
) {
	for key in input.just_pressed.iter() {
		enqueue_new_skill(key, queue, slots);
	}
}

fn enqueue_new_skill<TSlots: GetRef<SlotKey, Skill>, TQueue: Enqueue<(Skill, SlotKey)>>(
	key: &SlotKey,
	queue: &mut TQueue,
	slots: &TSlots,
) {
	let Some(skill) = slots.get(key).cloned() else {
		return;
	};
	queue.enqueue((skill, *key));
}

fn prime_skills<TQueue: IterMut<TQueuedSkill>, TQueuedSkill: Prime + Matches<SlotKey>>(
	input: &In<Input>,
	queue: &mut TQueue,
) {
	for key in input.just_released.iter() {
		prime_skill(key, queue);
	}
}

fn prime_skill<TQueue: IterMut<TQueuedSkill>, TQueuedSkill: Prime + Matches<SlotKey>>(
	key: &SlotKey,
	queue: &mut TQueue,
) {
	for skill in get_queued_skill(key, queue) {
		skill.prime();
	}
}

fn get_queued_skill<'a, TQueue: IterMut<TQueuedSkill>, TQueuedSkill: 'a + Matches<SlotKey>>(
	key: &'a SlotKey,
	queue: &'a mut TQueue,
) -> impl Iterator<Item = &'a mut TQueuedSkill> {
	queue.iter_mut().filter(move |skill| skill.matches(key))
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		ecs::system::{IntoSystem, Res, Resource},
		prelude::default,
	};
	use common::{
		components::Side,
		simple_init,
		test_tools::utils::SingleThreadedApp,
		traits::{mock::Mock, nested_mock::NestedMocks},
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
		impl Matches<SlotKey> for _SkillQueued {
			fn matches(&self, slot_key: &SlotKey) -> bool;
		}
	}

	simple_init!(Mock_SkillQueued);

	#[derive(Component, Default)]
	struct _Skills(HashMap<SlotKey, Skill>);

	impl GetRef<SlotKey, Skill> for _Skills {
		fn get<'a>(&'a self, key: &SlotKey) -> Option<&'a Skill> {
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

	impl IterMut<Mock_SkillQueued> for _Enqueue {
		fn iter_mut<'a>(&'a mut self) -> impl DoubleEndedIterator<Item = &'a mut Mock_SkillQueued>
		where
			Mock_SkillQueued: 'a,
		{
			self.queued.iter_mut()
		}
	}

	struct _SkillLoader;

	fn setup<TEnqueue: Enqueue<(Skill, SlotKey)> + IterMut<Mock_SkillQueued> + Component>() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<_Input>();

		app.add_systems(
			Update,
			(move |input: Res<_Input>| input.0.clone())
				.pipe(enqueue::<_Skills, TEnqueue, Mock_SkillQueued>),
		);

		app
	}

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

		impl IterMut<Mock_SkillQueued> for _Enqueue {
			fn iter_mut<'a>(
				&'a mut self,
			) -> impl DoubleEndedIterator<Item = &'a mut Mock_SkillQueued>
			where
				Mock_SkillQueued: 'a,
			{
				unsafe { EMPTY.iter_mut() }
			}
		}

		let mut app = setup::<_Enqueue>();

		let skills = _Skills(HashMap::from([(
			SlotKey::BottomHand(Side::Right),
			Skill {
				name: "my skill".to_owned(),
				..default()
			},
		)]));
		app.world_mut().spawn((
			skills,
			_Enqueue::new().with_mock(|mock| {
				mock.expect_enqueue()
					.times(1)
					.with(eq((
						Skill {
							name: "my skill".to_owned(),
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
	fn prime_skill() {
		let mut app = setup::<_Enqueue>();
		app.world_mut().spawn((
			_Skills::default(),
			_Enqueue {
				queued: vec![Mock_SkillQueued::new_mock(|mock| {
					mock.expect_prime().times(1).return_const(());
					mock.expect_matches().return_const(true);
				})],
			},
		));
		app.world_mut().resource_mut::<_Input>().0.just_released =
			vec![SlotKey::BottomHand(Side::Right)];

		app.update();
	}

	#[test]
	fn prime_skill_matching_with_key() {
		let mut app = setup::<_Enqueue>();
		app.world_mut().spawn((
			_Skills::default(),
			_Enqueue {
				queued: vec![
					Mock_SkillQueued::new_mock(|mock| {
						mock.expect_matches()
							.with(eq(SlotKey::BottomHand(Side::Right)))
							.return_const(true);
						mock.expect_prime().times(1).return_const(());
					}),
					Mock_SkillQueued::new_mock(|mock| {
						mock.expect_matches()
							.with(eq(SlotKey::BottomHand(Side::Right)))
							.return_const(false);
						mock.expect_prime().never().return_const(());
					}),
				],
			},
		));
		app.world_mut().resource_mut::<_Input>().0.just_released =
			vec![SlotKey::BottomHand(Side::Right)];

		app.update();
	}

	#[test]
	fn prime_all_in_queue() {
		let mut app = setup::<_Enqueue>();
		app.world_mut().spawn((
			_Skills::default(),
			_Enqueue {
				queued: vec![
					Mock_SkillQueued::new_mock(|mock| {
						mock.expect_prime().times(1).return_const(());
						mock.expect_matches().return_const(true);
					}),
					Mock_SkillQueued::new_mock(|mock| {
						mock.expect_prime().times(1).return_const(());
						mock.expect_matches().return_const(true);
					}),
				],
			},
		));
		app.world_mut().resource_mut::<_Input>().0.just_released =
			vec![SlotKey::BottomHand(Side::Right)];

		app.update();
	}
}
