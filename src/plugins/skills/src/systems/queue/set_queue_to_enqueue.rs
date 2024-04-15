use crate::components::queue::Queue;
use bevy::ecs::system::Query;

pub(crate) fn set_queue_to_enqueue(mut queues: Query<&mut Queue>) {
	for mut queue in &mut queues {
		let Queue::Dequeue(dequeue) = queue.as_ref() else {
			continue;
		};
		*queue = Queue::Enqueue(dequeue.clone().into());
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{
			queue::{DequeueAble, EnqueueAble, Queue, QueueCollection},
			SlotKey,
		},
		skill::{Queued, Skill},
	};
	use bevy::{
		app::{App, Update},
		utils::default,
	};
	use common::{components::Side, test_tools::utils::SingleThreadedApp};

	type _Queue = Queue<QueueCollection<EnqueueAble>, QueueCollection<DequeueAble>>;

	fn setup() -> App {
		let mut app = App::new_single_threaded([Update]);
		app.add_systems(Update, set_queue_to_enqueue);

		app
	}

	#[test]
	fn set_dequeue_to_enqueue() {
		let mut app = setup();
		let agent = app
			.world
			.spawn(_Queue::Dequeue(QueueCollection::new([Skill {
				name: "A",
				data: Queued {
					slot_key: SlotKey::Hand(Side::Main),
					..default()
				},
				..default()
			}])))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&_Queue::Enqueue(QueueCollection::new([Skill {
				name: "A",
				data: Queued {
					slot_key: SlotKey::Hand(Side::Main),
					..default()
				},
				..default()
			}]))),
			agent.get::<_Queue>()
		);
	}

	#[test]
	fn leave_enqueue_as_enqueue() {
		let mut app = setup();
		let agent = app
			.world
			.spawn(_Queue::Enqueue(QueueCollection::new([Skill {
				name: "A",
				data: Queued {
					slot_key: SlotKey::Hand(Side::Main),
					..default()
				},
				..default()
			}])))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&_Queue::Enqueue(QueueCollection::new([Skill {
				name: "A",
				data: Queued {
					slot_key: SlotKey::Hand(Side::Main),
					..default()
				},
				..default()
			}]))),
			agent.get::<_Queue>()
		);
	}
}
