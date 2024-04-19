use super::get_inputs::Input;
use crate::{
	components::SlotKey,
	traits::{Enqueue, IterMutWithKeys, Prime},
};
use bevy::ecs::{
	component::Component,
	system::{In, Query},
};
use common::traits::look_up::LookUp;

pub(crate) fn enqueue<
	TSkillSource: LookUp<SlotKey, TSourceSkill> + Component,
	TSourceSkill: Clone,
	TQueue: Enqueue<(TSourceSkill, SlotKey)> + IterMutWithKeys<SlotKey, TSkill> + Component,
	TSkill: Prime,
>(
	input: In<Input>,
	mut agents: Query<(&TSkillSource, &mut TQueue)>,
) {
	for (skills, mut queue) in &mut agents {
		let queue = queue.as_mut();
		enqueue_new_skills(&input, queue, skills);
		prime_skills(&input, queue);
	}
}

fn enqueue_new_skills<
	TSkillSource: LookUp<SlotKey, TSourceSkill>,
	TSourceSkill: Clone,
	TQueue: Enqueue<(TSourceSkill, SlotKey)>,
>(
	input: &In<Input>,
	queue: &mut TQueue,
	skills: &TSkillSource,
) {
	for key in input.just_pressed.iter() {
		enqueue_new_skill(key, skills, queue);
	}
}

fn enqueue_new_skill<
	TSkillSource: LookUp<SlotKey, TSourceSkill>,
	TSourceSkill: Clone,
	TQueue: Enqueue<(TSourceSkill, SlotKey)>,
>(
	key: &SlotKey,
	skills: &TSkillSource,
	queue: &mut TQueue,
) {
	let Some(skill) = skills.get(key).cloned() else {
		return;
	};
	queue.enqueue((skill, *key));
}

fn prime_skills<TQueue: IterMutWithKeys<SlotKey, TSkill>, TSkill: Prime>(
	input: &In<Input>,
	queue: &mut TQueue,
) {
	for key in input.just_released.iter() {
		prime_skill(key, queue);
	}
}

fn prime_skill<TQueue: IterMutWithKeys<SlotKey, TSkill>, TSkill: Prime>(
	key: &SlotKey,
	queue: &mut TQueue,
) {
	for (.., skill) in get_queued_skill(key, queue) {
		skill.prime();
	}
}

fn get_queued_skill<'a, TQueue: IterMutWithKeys<SlotKey, TSkill>, TSkill: 'a>(
	key: &'a SlotKey,
	queue: &'a mut TQueue,
) -> impl Iterator<Item = (SlotKey, &'a mut TSkill)> {
	queue
		.iter_mut_with_keys()
		.filter(move |(queued_key, ..)| queued_key == key)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::SlotKey;
	use bevy::{
		app::{App, Update},
		ecs::system::{IntoSystem, Res, Resource},
		utils::default,
	};
	use common::{components::Side, test_tools::utils::SingleThreadedApp};
	use mockall::{automock, mock, predicate::eq};
	use std::collections::HashMap;

	#[derive(Resource, Default)]
	struct _Input(Input);

	#[derive(Clone, Debug, PartialEq)]
	struct _Skill(u32);

	mock! {
		_SkillQueued {}
		impl Prime for _SkillQueued {
			fn prime(&mut self) {}
		}
	}

	#[derive(Component, Default)]
	struct _Skills(HashMap<SlotKey, _Skill>);

	impl LookUp<SlotKey, _Skill> for _Skills {
		fn get<'a>(&'a self, key: &SlotKey) -> Option<&'a _Skill> {
			self.0.get(key)
		}
	}

	#[derive(Component, Default)]
	struct _Enqueue {
		mock: Mock_Enqueue,
		queued: Vec<(SlotKey, Mock_SkillQueued)>,
	}

	#[automock]
	impl Enqueue<(_Skill, SlotKey)> for _Enqueue {
		fn enqueue(&mut self, item: (_Skill, SlotKey)) {
			self.mock.enqueue(item)
		}
	}

	impl IterMutWithKeys<SlotKey, Mock_SkillQueued> for _Enqueue {
		fn iter_mut_with_keys<'a>(
			&'a mut self,
		) -> impl DoubleEndedIterator<Item = (SlotKey, &'a mut Mock_SkillQueued)>
		where
			Mock_SkillQueued: 'a,
		{
			self.queued.iter_mut().map(|(k, s)| (*k, s))
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<_Input>();
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

		let skills = _Skills(HashMap::from([(SlotKey::Hand(Side::Main), _Skill(121))]));
		let mut enqueue = _Enqueue::default();

		enqueue
			.mock
			.expect_enqueue()
			.times(1)
			.with(eq((_Skill(121), SlotKey::Hand(Side::Main))))
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

		app.world.spawn((
			_Skills::default(),
			_Enqueue {
				queued: vec![(SlotKey::Hand(Side::Main), skill)],
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

		let mut mismatched_skill = Mock_SkillQueued::default();
		mismatched_skill.expect_prime().never().return_const(());

		app.world.spawn((
			_Skills::default(),
			_Enqueue {
				queued: vec![
					(SlotKey::Hand(Side::Main), skill),
					(SlotKey::Hand(Side::Off), mismatched_skill),
				],
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

		let mut skill_b = Mock_SkillQueued::default();
		skill_b.expect_prime().times(1).return_const(());

		app.world.spawn((
			_Skills::default(),
			_Enqueue {
				queued: vec![
					(SlotKey::Hand(Side::Main), skill_a),
					(SlotKey::Hand(Side::Main), skill_b),
				],
				..default()
			},
		));

		app.world.resource_mut::<_Input>().0.just_released = vec![SlotKey::Hand(Side::Main)];
		app.update();
	}
}
