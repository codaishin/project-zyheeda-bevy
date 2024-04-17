use super::get_inputs::Input;
use crate::{
	components::SlotKey,
	skill::{Activation, Queued, Skill},
	traits::{Enqueue, IterMut},
};
use bevy::ecs::{
	component::Component,
	system::{In, Query},
};
use common::traits::look_up::LookUp;

pub(crate) fn enqueue_skills<
	TSkillSource: LookUp<SlotKey, Skill> + Component,
	TEnqueue: Enqueue<(Skill, SlotKey)> + IterMut<Skill<Queued>> + Component,
>(
	input: In<Input>,
	mut agents: Query<(&TSkillSource, &mut TEnqueue)>,
) {
	for (skills, mut queue) in &mut agents {
		let queue = queue.as_mut();
		enqueue_new_skills(&input, queue, skills);
		prime_skills(&input, queue);
	}
}

fn enqueue_new_skills<TSkillSource: LookUp<SlotKey, Skill>, TEnqueue: Enqueue<(Skill, SlotKey)>>(
	input: &In<Input>,
	queue: &mut TEnqueue,
	skills: &TSkillSource,
) {
	for key in input.just_pressed.iter() {
		enqueue_new_skill(key, skills, queue);
	}
}

fn enqueue_new_skill<TSkillSource: LookUp<SlotKey, Skill>, TQueue: Enqueue<(Skill, SlotKey)>>(
	key: &SlotKey,
	skills: &TSkillSource,
	queue: &mut TQueue,
) {
	let Some(skill) = skills.get(key).cloned() else {
		return;
	};
	queue.enqueue((skill, *key));
}

fn prime_skills<TQueue: IterMut<Skill<Queued>>>(input: &In<Input>, queue: &mut TQueue) {
	for key in input.just_released.iter() {
		prime_skill(key, queue);
	}
}

fn prime_skill<TQueue: IterMut<Skill<Queued>>>(key: &SlotKey, queue: &mut TQueue) {
	for skill in get_queued_skill(key, queue) {
		skill.data.mode = Activation::Primed;
	}
}

fn get_queued_skill<'a, TQueue: IterMut<Skill<Queued>>>(
	key: &'a SlotKey,
	queue: &'a mut TQueue,
) -> impl Iterator<Item = &'a mut Skill<Queued>> {
	queue
		.iter_mut()
		.filter(move |skill| &skill.data.slot_key == key)
}

#[cfg(test)]
mod tests {
	use std::collections::HashMap;

	use super::*;
	use crate::{
		components::SlotKey,
		skill::{Queued, Skill},
	};
	use bevy::{
		app::{App, Update},
		ecs::system::{IntoSystem, Res, Resource},
		utils::default,
	};
	use common::{components::Side, test_tools::utils::SingleThreadedApp};
	use mockall::{automock, predicate::eq};

	#[derive(Resource, Default)]
	struct _Input(Input);

	#[derive(Component, Default)]
	struct _Skills(HashMap<SlotKey, Skill>);

	impl LookUp<SlotKey, Skill> for _Skills {
		fn get<'a>(&'a self, key: &SlotKey) -> Option<&'a Skill> {
			self.0.get(key)
		}
	}

	#[derive(Component, Default)]
	struct _Enqueue {
		mock: Mock_Enqueue,
		queued: Vec<Skill<Queued>>,
	}

	#[automock]
	impl Enqueue<(Skill, SlotKey)> for _Enqueue {
		fn enqueue(&mut self, item: (Skill, SlotKey)) {
			self.mock.enqueue(item)
		}
	}

	impl IterMut<Skill<Queued>> for _Enqueue {
		fn iter_mut<'a>(&'a mut self) -> impl DoubleEndedIterator<Item = &'a mut Skill<Queued>>
		where
			Skill<Queued>: 'a,
		{
			self.queued.iter_mut()
		}
	}

	fn setup() -> App {
		let mut app = App::new_single_threaded([Update]);
		app.init_resource::<_Input>();
		app.add_systems(
			Update,
			(move |input: Res<_Input>| input.0.clone()).pipe(enqueue_skills::<_Skills, _Enqueue>),
		);

		app
	}

	#[test]
	fn enqueue_skill() {
		let mut app = setup();
		let slot = SlotKey::Hand(Side::Main);
		let skill = Skill {
			name: "my skill",
			..default()
		};
		let mut enqueue = _Enqueue::default();

		enqueue
			.mock
			.expect_enqueue()
			.times(1)
			.with(eq((skill.clone(), slot)))
			.return_const(());

		app.world
			.spawn((_Skills(HashMap::from([(slot, skill)])), enqueue));

		app.world.resource_mut::<_Input>().0.just_pressed = vec![slot];
		app.update();
	}

	#[test]
	fn prime_skill() {
		let mut app = setup();
		let agent = app
			.world
			.spawn((
				_Skills::default(),
				_Enqueue {
					queued: vec![Skill {
						name: "a",
						data: Queued {
							slot_key: SlotKey::Hand(Side::Main),
							mode: Activation::Primed,
						},
						..default()
					}],
					..default()
				},
			))
			.id();

		app.world.resource_mut::<_Input>().0.just_pressed = vec![];
		app.world.resource_mut::<_Input>().0.just_released = vec![SlotKey::Hand(Side::Main)];
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&vec![Skill {
				name: "a",
				data: Queued {
					slot_key: SlotKey::Hand(Side::Main),
					mode: Activation::Primed,
				},
				..default()
			}]),
			agent.get::<_Enqueue>().map(|e| &e.queued),
		);
	}

	#[test]
	fn prime_skill_matching_with_key() {
		let mut app = setup();
		let agent = app
			.world
			.spawn((
				_Skills::default(),
				_Enqueue {
					queued: vec![
						Skill {
							name: "a",
							data: Queued {
								slot_key: SlotKey::Hand(Side::Off),
								..default()
							},
							..default()
						},
						Skill {
							name: "b",
							data: Queued {
								slot_key: SlotKey::Hand(Side::Main),
								..default()
							},
							..default()
						},
					],
					..default()
				},
			))
			.id();

		app.world.resource_mut::<_Input>().0.just_pressed = vec![];
		app.world.resource_mut::<_Input>().0.just_released = vec![SlotKey::Hand(Side::Main)];
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&vec![
				Skill {
					name: "a",
					data: Queued {
						slot_key: SlotKey::Hand(Side::Off),
						..default()
					},
					..default()
				},
				Skill {
					name: "b",
					data: Queued {
						slot_key: SlotKey::Hand(Side::Main),
						mode: Activation::Primed,
					},
					..default()
				},
			]),
			agent.get::<_Enqueue>().map(|e| &e.queued)
		);
	}

	#[test]
	fn prime_all_in_queue() {
		let mut app = setup();
		let agent = app
			.world
			.spawn((
				_Skills::default(),
				_Enqueue {
					queued: vec![
						Skill {
							name: "a",
							data: Queued {
								slot_key: SlotKey::Hand(Side::Main),
								..default()
							},
							..default()
						},
						Skill {
							name: "b",
							data: Queued {
								slot_key: SlotKey::Hand(Side::Main),
								..default()
							},
							..default()
						},
					],
					..default()
				},
			))
			.id();

		app.world.resource_mut::<_Input>().0.just_pressed = vec![];
		app.world.resource_mut::<_Input>().0.just_released = vec![SlotKey::Hand(Side::Main)];
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&vec![
				Skill {
					name: "a",
					data: Queued {
						slot_key: SlotKey::Hand(Side::Main),
						mode: Activation::Primed,
					},
					..default()
				},
				Skill {
					name: "b",
					data: Queued {
						slot_key: SlotKey::Hand(Side::Main),
						mode: Activation::Primed,
					},
					..default()
				},
			]),
			agent.get::<_Enqueue>().map(|e| &e.queued)
		);
	}
}
