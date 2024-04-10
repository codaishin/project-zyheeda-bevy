use crate::components::queue::Queue;
use bevy::ecs::system::Query;

pub(crate) fn set_queue_to_dequeue(mut queues: Query<&mut Queue>) {
	for mut queue in &mut queues {
		let Queue::Enqueue(enqueue) = queue.as_ref() else {
			continue;
		};
		*queue = Queue::Dequeue(enqueue.clone().into());
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{
			queue::{Queue as GenericQueue, QueueCollection},
			SlotKey,
		},
		skill::{Queued, Skill},
	};
	use bevy::{
		app::{App, Update},
		utils::default,
	};
	use common::{components::Side, test_tools::utils::SingleThreadedApp};

	type Queue = GenericQueue;

	fn setup() -> App {
		let mut app = App::new_single_threaded([Update]);
		app.add_systems(Update, set_queue_to_dequeue);

		app
	}

	#[test]
	fn set_enqueue_to_dequeue() {
		let mut app = setup();
		let agent = app
			.world
			.spawn(Queue::Enqueue(QueueCollection::new([Skill {
				name: "A",
				data: Queued(SlotKey::Hand(Side::Main)),
				..default()
			}])))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&Queue::Dequeue(QueueCollection::new([Skill {
				name: "A",
				data: Queued(SlotKey::Hand(Side::Main)),
				..default()
			}]))),
			agent.get::<Queue>()
		);
	}

	#[test]
	fn leave_dequeue_as_dequeue() {
		let mut app = setup();
		let agent = app
			.world
			.spawn(Queue::Dequeue(QueueCollection::new([Skill {
				name: "A",
				data: Queued(SlotKey::Hand(Side::Main)),
				..default()
			}])))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&Queue::Dequeue(QueueCollection::new([Skill {
				name: "A",
				data: Queued(SlotKey::Hand(Side::Main)),
				..default()
			}]))),
			agent.get::<Queue>()
		);
	}
}
