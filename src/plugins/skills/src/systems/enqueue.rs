use super::get_inputs::Input;
use crate::{
	items::slot_key::SlotKey,
	traits::{Enqueue, IterMut, Matches, Prime},
};
use bevy::{
	asset::{Asset, Assets, Handle},
	ecs::{
		component::Component,
		system::{In, Query, Res},
	},
};
use common::traits::get::Get;

pub(crate) fn enqueue<
	TSlots: Get<SlotKey, Handle<TSkill>> + Component,
	TSkill: Asset + Clone,
	TQueue: Enqueue<(TSkill, SlotKey)> + IterMut<TQueuedSkill> + Component,
	TQueuedSkill: Prime + Matches<SlotKey>,
>(
	input: In<Input>,
	mut agents: Query<(&TSlots, &mut TQueue)>,
	skills: Res<Assets<TSkill>>,
) {
	for (slots, mut queue) in &mut agents {
		let queue = queue.as_mut();
		enqueue_new_skills(&input, queue, slots, &skills);
		prime_skills(&input, queue);
	}
}

fn enqueue_new_skills<
	TSlots: Get<SlotKey, Handle<TSkill>>,
	TSkill: Asset + Clone,
	TQueue: Enqueue<(TSkill, SlotKey)>,
>(
	input: &In<Input>,
	queue: &mut TQueue,
	slots: &TSlots,
	skills: &Assets<TSkill>,
) {
	for key in input.just_pressed.iter() {
		enqueue_new_skill(key, queue, slots, skills);
	}
}

fn enqueue_new_skill<
	TSlots: Get<SlotKey, Handle<TSkill>>,
	TSkill: Asset + Clone,
	TQueue: Enqueue<(TSkill, SlotKey)>,
>(
	key: &SlotKey,
	queue: &mut TQueue,
	slots: &TSlots,
	skills: &Assets<TSkill>,
) {
	let Some(skill) = slots.get(key).cloned() else {
		return;
	};
	let Some(skill) = skills.get(skill).cloned() else {
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
		asset::Handle,
		ecs::system::{IntoSystem, Res, Resource},
		prelude::default,
		reflect::TypePath,
	};
	use common::{components::Side, test_tools::utils::SingleThreadedApp};
	use mockall::{automock, mock, predicate::eq};
	use std::collections::HashMap;

	#[derive(Resource, Default)]
	struct _Input(Input);

	#[derive(Debug, PartialEq, TypePath, Asset, Clone)]
	struct _Skill(&'static str);

	mock! {
		_SkillQueued {}
		impl Prime for _SkillQueued {
			fn prime(&mut self) {}
		}
		impl Matches<SlotKey> for _SkillQueued {
			fn matches(&self, slot_key: &SlotKey) -> bool;
		}
	}

	#[derive(Component, Default)]
	struct _Skills(HashMap<SlotKey, Handle<_Skill>>);

	impl Get<SlotKey, Handle<_Skill>> for _Skills {
		fn get<'a>(&'a self, key: &SlotKey) -> Option<&'a Handle<_Skill>> {
			self.0.get(key)
		}
	}

	#[derive(Component, Default)]
	struct _Enqueue {
		mock: Mock_Enqueue,
		queued: Vec<Mock_SkillQueued>,
	}

	#[automock]
	impl Enqueue<(_Skill, SlotKey)> for _Enqueue {
		fn enqueue(&mut self, item: (_Skill, SlotKey)) {
			self.mock.enqueue(item)
		}
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

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<_Input>();
		app.init_resource::<Assets<_Skill>>();

		app.add_systems(
			Update,
			(move |input: Res<_Input>| input.0.clone())
				.pipe(enqueue::<_Skills, _Skill, _Enqueue, Mock_SkillQueued>),
		);

		app
	}

	#[test]
	fn enqueue_skill_from_skills() {
		let mut app = setup();

		let skill = app
			.world
			.resource_mut::<Assets<_Skill>>()
			.add(_Skill("my skill"));

		let skills = _Skills(HashMap::from([(SlotKey::Hand(Side::Main), skill)]));
		let mut enqueue = _Enqueue::default();

		enqueue
			.mock
			.expect_enqueue()
			.times(1)
			.with(eq((_Skill("my skill"), SlotKey::Hand(Side::Main))))
			.return_const(());

		app.world.spawn((skills, enqueue));

		app.world.resource_mut::<_Input>().0.just_pressed = vec![SlotKey::Hand(Side::Main)];
		app.update();
	}

	#[test]
	fn prime_skill() {
		let mut app = setup();
		let mut skill = Mock_SkillQueued::default();
		skill.expect_prime().times(1).return_const(());
		skill.expect_matches().return_const(true);

		app.world.spawn((
			_Skills::default(),
			_Enqueue {
				queued: vec![skill],
				..default()
			},
		));

		app.world.resource_mut::<_Input>().0.just_released = vec![SlotKey::Hand(Side::Main)];
		app.update();
	}

	#[test]
	fn prime_skill_matching_with_key() {
		let mut app = setup();

		let mut skill = Mock_SkillQueued::default();
		skill.expect_prime().return_const(());
		skill
			.expect_matches()
			.times(1)
			.with(eq(SlotKey::Hand(Side::Main)))
			.return_const(true);

		let mut mismatched_skill = Mock_SkillQueued::default();
		mismatched_skill.expect_prime().never().return_const(());
		mismatched_skill
			.expect_matches()
			.times(1)
			.with(eq(SlotKey::Hand(Side::Main)))
			.return_const(false);

		app.world.spawn((
			_Skills::default(),
			_Enqueue {
				queued: vec![skill, mismatched_skill],
				..default()
			},
		));

		app.world.resource_mut::<_Input>().0.just_released = vec![SlotKey::Hand(Side::Main)];
		app.update();
	}

	#[test]
	fn prime_all_in_queue() {
		let mut app = setup();

		let mut skill_a = Mock_SkillQueued::default();
		skill_a.expect_prime().times(1).return_const(());
		skill_a.expect_matches().return_const(true);

		let mut skill_b = Mock_SkillQueued::default();
		skill_b.expect_prime().times(1).return_const(());
		skill_b.expect_matches().return_const(true);

		app.world.spawn((
			_Skills::default(),
			_Enqueue {
				queued: vec![skill_a, skill_b],
				..default()
			},
		));

		app.world.resource_mut::<_Input>().0.just_released = vec![SlotKey::Hand(Side::Main)];
		app.update();
	}
}
