use crate::{
	components::queue::{EnqueueAble, Queue, QueueCollection},
	skill::{Queued, Skill},
	traits::TryDequeue,
	Error,
};
use bevy::prelude::{Entity, Query};
use common::errors::Level;

type Components<'a, TDequeue> = (
	Entity,
	&'a mut Queue<QueueCollection<EnqueueAble>, TDequeue>,
);

fn no_dequeue_mode(id: Entity) -> String {
	format!("{id:?}: Attempted dequeue on a queue set to enqueue")
}

pub(crate) fn dequeue<TDequeue: TryDequeue<Skill<Queued>> + Send + Sync + 'static>(
	mut agents: Query<Components<TDequeue>>,
) -> Vec<Result<(), Error>> {
	agents
		.iter_mut()
		.map(|(agent, mut queue)| dequeue_to_active(&mut queue, agent))
		.collect()
}

fn dequeue_to_active<TDequeue: TryDequeue<Skill<Queued>>>(
	queue: &mut Queue<QueueCollection<EnqueueAble>, TDequeue>,
	agent: Entity,
) -> Result<(), Error> {
	let Queue::Dequeue(queue) = queue else {
		return Err(Error {
			msg: no_dequeue_mode(agent),
			lvl: Level::Error,
		});
	};

	queue.try_dequeue();

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::skill::{Queued, Skill};
	use bevy::{
		ecs::system::IntoSystem,
		prelude::{App, Update},
	};
	use common::{
		errors::Level,
		systems::log::test_tools::{fake_log_error_many_recourse, FakeErrorLogManyResource},
		test_tools::utils::SingleThreadedApp,
	};
	use mockall::mock;

	mock! {
		_Dequeue {}
		impl TryDequeue<Skill<Queued>> for _Dequeue {
			fn try_dequeue(&mut self) {}
		}
	}

	type _TestQueue = Queue<QueueCollection<EnqueueAble>, Mock_Dequeue>;

	fn setup() -> App {
		let mut app = App::new_single_threaded([Update]);
		app.add_systems(
			Update,
			dequeue::<Mock_Dequeue>.pipe(fake_log_error_many_recourse),
		);

		app
	}

	#[test]
	fn try_dequeue() {
		let mut app = setup();
		let mut queue = Mock_Dequeue::default();
		queue.expect_try_dequeue().times(1).return_const(());

		app.world.spawn(_TestQueue::Dequeue(queue));

		app.update();
	}

	#[test]
	fn error_when_queue_not_in_dequeue_state() {
		let mut app = setup();
		let agent = app
			.world
			.spawn(_TestQueue::Enqueue(QueueCollection::new([])))
			.id();

		app.update();

		assert_eq!(
			Some(&FakeErrorLogManyResource(vec![Error {
				lvl: Level::Error,
				msg: no_dequeue_mode(agent),
			}])),
			app.world.get_resource::<FakeErrorLogManyResource>()
		);
	}

	#[test]
	fn no_error_when_queue_in_dequeue_state() {
		let mut app = setup();
		let mut queue = Mock_Dequeue::default();
		queue.expect_try_dequeue().return_const(());

		app.world.spawn(_TestQueue::Dequeue(queue));

		app.update();

		assert_eq!(None, app.world.get_resource::<FakeErrorLogManyResource>());
	}
}
