use super::get_inputs::Input;
use crate::{
	components::{SlotKey, Slots},
	skill::{Activation, Queued, Skill},
	traits::{Enqueue, IterMut},
};
use bevy::{
	ecs::{
		component::Component,
		system::{In, Query},
	},
	utils::default,
};

pub(crate) fn enqueue_skills<
	TEnqueue: Enqueue<Skill<Queued>> + IterMut<Skill<Queued>> + Component,
>(
	input: In<Input>,
	mut agents: Query<(&Slots, &mut TEnqueue)>,
) {
	for (slots, mut queue) in &mut agents {
		let queue = queue.as_mut();
		enqueue_new_skills(&input, queue, slots);
		prime_skills(&input, queue);
	}
}

fn enqueue_new_skills<TEnqueue: Enqueue<Skill<Queued>>>(
	input: &In<Input>,
	queue: &mut TEnqueue,
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
	use super::*;
	use crate::{
		components::{Item, Slot, SlotKey, Slots},
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
	use common::{components::Side, test_tools::utils::SingleThreadedApp};
	use std::collections::HashMap;

	#[derive(Resource, Default)]
	struct _Input(Input);

	#[derive(Component, Default, Debug, PartialEq)]
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

	fn setup() -> App {
		let mut app = App::new_single_threaded([Update]);
		app.init_resource::<_Input>();
		app.add_systems(
			Update,
			(move |input: Res<_Input>| input.0.clone()).pipe(enqueue_skills::<_Enqueue>),
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
					},
				)])),
				_Enqueue::default(),
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
			agent.get::<_Enqueue>()
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
					},
				)])),
				_Enqueue::default(),
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
			agent.get::<_Enqueue>(),
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
					},
				)])),
				_Enqueue {
					queued: vec![Skill {
						data: Queued {
							slot_key: SlotKey::Hand(Side::Off),
							..default()
						},
						..default()
					}],
				},
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
			agent.get::<_Enqueue>()
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
					},
				)])),
				_Enqueue {
					queued: vec![Skill {
						name: "other",
						data: Queued {
							slot_key: SlotKey::Hand(Side::Main),
							..default()
						},
						..default()
					}],
				},
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
			agent.get::<_Enqueue>()
		);
	}
}
