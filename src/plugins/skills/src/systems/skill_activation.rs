use super::get_inputs::Input;
use crate::{
	components::{queue::Queue, SlotKey, Slots},
	skill::{Activation, Queued, Skill},
	traits::{Enqueue, IterMut},
};
use bevy::{
	ecs::{
		entity::Entity,
		system::{In, Query},
	},
	utils::default,
};
use common::errors::Level;

type Components<'a, TEnqueue> = (Entity, &'a Slots, &'a mut Queue<TEnqueue>);
use crate::Error;

fn no_enqueue_mode(id: Entity) -> String {
	format!("{id:?}: Attempted enqueue on a queue set to dequeue")
}

pub(crate) fn skill_activation<
	TEnqueue: Enqueue<Skill<Queued>> + IterMut<Skill<Queued>> + Sync + Send + 'static,
>(
	input: In<Input>,
	mut agents: Query<Components<TEnqueue>>,
) -> Vec<Result<(), Error>> {
	agents
		.iter_mut()
		.map(|(id, slots, mut queue)| {
			let Queue::Enqueue(queue) = queue.as_mut() else {
				return Err(Error {
					msg: no_enqueue_mode(id),
					lvl: Level::Error,
				});
			};
			enqueue_new_skills(&input, queue, slots);
			prime_skills(&input, queue);
			Ok(())
		})
		.collect()
}

fn enqueue_new_skills<TQueue: Enqueue<Skill<Queued>>>(
	input: &In<Input>,
	queue: &mut TQueue,
	slots: &Slots,
) {
	for key in input.just_pressed.iter() {
		enqueue_new_skill(key, slots, queue);
	}
}

fn enqueue_new_skill<TQueue: Enqueue<Skill<Queued>>>(
	key: &SlotKey,
	slots: &Slots,
	queue: &mut TQueue,
) {
	let Some(skill) = get_slot_skill(key, slots) else {
		return;
	};
	queue.enqueue(skill.with(Queued {
		slot_key: *key,
		..default()
	}));
}

fn get_slot_skill(key: &SlotKey, slots: &Slots) -> Option<Skill> {
	slots
		.0
		.get(key)
		.and_then(|s| s.item.clone())
		.and_then(|i| i.skill)
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
		components::{queue::QueueCollection, Item, Slot, SlotKey, Slots},
		skill::{Queued, Skill},
	};
	use bevy::{
		app::{App, Update},
		ecs::{
			entity::Entity,
			system::{IntoSystem, Res, Resource},
		},
		utils::default,
	};
	use common::{
		components::Side,
		errors::Level,
		systems::log::test_tools::{fake_log_error_many_recourse, FakeErrorLogManyResource},
		test_tools::utils::SingleThreadedApp,
	};

	type _TestQueue = Queue<_Enqueue>;

	#[derive(Resource, Default)]
	struct _Input(Input);

	#[derive(Default, Debug, PartialEq)]
	struct _Enqueue {
		queued: Vec<Skill<Queued>>,
	}

	impl Enqueue<Skill<Queued>> for _Enqueue {
		fn enqueue(&mut self, item: Skill<Queued>) {
			self.queued.push(item)
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

	fn get_enqueue(queue: &_TestQueue) -> Option<&_Enqueue> {
		match queue {
			_TestQueue::Enqueue(queue) => Some(queue),
			_TestQueue::Dequeue(_) => None,
		}
	}

	fn setup() -> App {
		let mut app = App::new_single_threaded([Update]);
		app.init_resource::<_Input>();
		app.add_systems(
			Update,
			(move |input: Res<_Input>| input.0.clone())
				.pipe(skill_activation::<_Enqueue>)
				.pipe(fake_log_error_many_recourse),
		);

		app
	}

	#[test]
	fn enqueue_skill() {
		let mut app = setup();
		let agent = app
			.world
			.spawn((
				Slots(HashMap::from([(
					SlotKey::Hand(Side::Main),
					Slot {
						entity: Entity::from_raw(42),
						item: Some(Item {
							skill: Some(Skill {
								name: "my skill",
								..default()
							}),
							..default()
						}),
						combo_skill: None,
					},
				)])),
				_TestQueue::Enqueue(_Enqueue::default()),
			))
			.id();

		app.world.resource_mut::<_Input>().0.just_pressed = vec![SlotKey::Hand(Side::Main)];
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&_Enqueue {
				queued: vec![Skill {
					name: "my skill",
					data: Queued {
						slot_key: SlotKey::Hand(Side::Main),
						mode: Activation::Waiting,
					},
					..default()
				}]
			}),
			agent.get::<_TestQueue>().and_then(get_enqueue)
		);
	}

	#[test]
	fn prime_skill() {
		let mut app = setup();
		let agent = app
			.world
			.spawn((
				Slots(HashMap::from([(
					SlotKey::Hand(Side::Main),
					Slot {
						entity: Entity::from_raw(42),
						item: Some(Item {
							skill: Some(Skill {
								name: "main",
								..default()
							}),
							..default()
						}),
						combo_skill: None,
					},
				)])),
				_TestQueue::Enqueue(_Enqueue::default()),
			))
			.id();

		app.world.resource_mut::<_Input>().0.just_pressed = vec![SlotKey::Hand(Side::Main)];
		app.update();

		app.world.resource_mut::<_Input>().0.just_pressed = vec![];
		app.world.resource_mut::<_Input>().0.just_released = vec![SlotKey::Hand(Side::Main)];
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&_Enqueue {
				queued: vec![Skill {
					name: "main",
					data: Queued {
						slot_key: SlotKey::Hand(Side::Main),
						mode: Activation::Primed,
					},
					..default()
				}]
			}),
			agent.get::<_TestQueue>().and_then(get_enqueue),
		);
	}

	#[test]
	fn prime_skill_matching_with_key() {
		let mut app = setup();
		let agent = app
			.world
			.spawn((
				Slots(HashMap::from([(
					SlotKey::Hand(Side::Main),
					Slot {
						entity: Entity::from_raw(42),
						item: Some(Item {
							skill: Some(Skill {
								name: "main",
								..default()
							}),
							..default()
						}),
						combo_skill: None,
					},
				)])),
				_TestQueue::Enqueue(_Enqueue {
					queued: vec![Skill {
						data: Queued {
							slot_key: SlotKey::Hand(Side::Off),
							..default()
						},
						..default()
					}],
				}),
			))
			.id();

		app.world.resource_mut::<_Input>().0.just_pressed = vec![SlotKey::Hand(Side::Main)];
		app.update();

		app.world.resource_mut::<_Input>().0.just_pressed = vec![];
		app.world.resource_mut::<_Input>().0.just_released = vec![SlotKey::Hand(Side::Main)];
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&_Enqueue {
				queued: vec![
					Skill {
						data: Queued {
							slot_key: SlotKey::Hand(Side::Off),
							..default()
						},
						..default()
					},
					Skill {
						name: "main",
						data: Queued {
							slot_key: SlotKey::Hand(Side::Main),
							mode: Activation::Primed,
						},
						..default()
					}
				]
			}),
			agent.get::<_TestQueue>().and_then(get_enqueue)
		);
	}

	#[test]
	fn prime_all_in_queue() {
		let mut app = setup();
		let agent = app
			.world
			.spawn((
				Slots(HashMap::from([(
					SlotKey::Hand(Side::Main),
					Slot {
						entity: Entity::from_raw(42),
						item: Some(Item {
							skill: Some(Skill {
								name: "main",
								..default()
							}),
							..default()
						}),
						combo_skill: None,
					},
				)])),
				_TestQueue::Enqueue(_Enqueue {
					queued: vec![Skill {
						name: "other",
						data: Queued {
							slot_key: SlotKey::Hand(Side::Main),
							..default()
						},
						..default()
					}],
				}),
			))
			.id();

		app.world.resource_mut::<_Input>().0.just_pressed = vec![SlotKey::Hand(Side::Main)];
		app.update();

		app.world.resource_mut::<_Input>().0.just_pressed = vec![];
		app.world.resource_mut::<_Input>().0.just_released = vec![SlotKey::Hand(Side::Main)];
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&_Enqueue {
				queued: vec![
					Skill {
						name: "other",
						data: Queued {
							slot_key: SlotKey::Hand(Side::Main),
							mode: Activation::Primed,
						},
						..default()
					},
					Skill {
						name: "main",
						data: Queued {
							slot_key: SlotKey::Hand(Side::Main),
							mode: Activation::Primed,
						},
						..default()
					},
				]
			}),
			agent.get::<_TestQueue>().and_then(get_enqueue)
		);
	}

	#[test]
	fn error_when_queue_not_in_enqueue_state() {
		let mut app = setup();
		let agent = app
			.world
			.spawn((
				Slots(HashMap::from([(
					SlotKey::Hand(Side::Main),
					Slot {
						entity: Entity::from_raw(42),
						item: Some(Item {
							skill: Some(Skill::default()),
							..default()
						}),
						combo_skill: None,
					},
				)])),
				_TestQueue::Dequeue(QueueCollection::new([])),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&FakeErrorLogManyResource(vec![Error {
				msg: no_enqueue_mode(agent),
				lvl: Level::Error
			}])),
			app.world.get_resource::<FakeErrorLogManyResource>()
		);
	}

	#[test]
	fn no_error_when_queue_in_enqueue_state() {
		let mut app = setup();
		app.world.spawn((
			Slots(HashMap::from([(
				SlotKey::Hand(Side::Main),
				Slot {
					entity: Entity::from_raw(42),
					item: Some(Item {
						skill: Some(Skill::default()),
						..default()
					}),
					combo_skill: None,
				},
			)])),
			_TestQueue::Enqueue(_Enqueue::default()),
		));

		app.update();

		assert_eq!(None, app.world.get_resource::<FakeErrorLogManyResource>());
	}
}
