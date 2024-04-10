use crate::{
	components::{
		queue::{EnqueueAble, Queue, QueueCollection},
		Track,
	},
	skill::{Active, Queued, Skill},
	traits::Dequeue,
	Error,
};
use bevy::{
	ecs::query::Without,
	prelude::{Commands, Entity, Query},
};
use common::errors::Level;

type Components<'a, TDequeue> = (
	Entity,
	&'a mut Queue<QueueCollection<EnqueueAble>, TDequeue>,
);

fn no_dequeue_mode(id: Entity) -> String {
	format!("{id:?}: Attempted dequeue on a queue set to enqueue")
}

pub(crate) fn dequeue<TDequeue: Dequeue<Skill<Queued>> + Send + Sync + 'static>(
	mut commands: Commands,
	mut agents: Query<Components<TDequeue>, Without<Track<Skill<Active>>>>,
) -> Vec<Result<(), Error>> {
	agents
		.iter_mut()
		.map(|(agent, mut queue)| dequeue_to_active(&mut commands, &mut queue, agent))
		.collect()
}

fn dequeue_to_active<TDequeue: Dequeue<Skill<Queued>>>(
	commands: &mut Commands,
	queue: &mut Queue<QueueCollection<EnqueueAble>, TDequeue>,
	agent: Entity,
) -> Result<(), Error> {
	let Queue::Dequeue(queue) = queue else {
		return Err(Error {
			msg: no_dequeue_mode(agent),
			lvl: Level::Error,
		});
	};

	let Some(skill) = queue.dequeue() else {
		return Ok(());
	};

	let Some(mut agent) = commands.get_entity(agent) else {
		return Ok(());
	};

	agent.try_insert(Track::new(skill.to_active()));

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::SlotKey,
		skill::{Active, Cast, Queued, Skill},
	};
	use bevy::{
		ecs::system::IntoSystem,
		prelude::{default, App, Update},
	};
	use common::{
		components::Side,
		errors::Level,
		systems::log::test_tools::{fake_log_error_many_recourse, FakeErrorLogManyResource},
		test_tools::utils::SingleThreadedApp,
	};
	use mockall::mock;
	use std::time::Duration;

	mock! {
		_Dequeue {}
		impl Dequeue<Skill<Queued>> for _Dequeue {
			fn dequeue(&mut self) -> Option<Skill<Queued>> {}
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
	fn dequeue_behavior_to_agent() {
		let mut app = setup();
		let mut queue = Mock_Dequeue::default();
		queue.expect_dequeue().return_const(Some(Skill {
			cast: Cast {
				pre: Duration::from_millis(42),
				..default()
			},
			data: Queued(SlotKey::SkillSpawn),
			..default()
		}));
		let agent = app.world.spawn(_TestQueue::Dequeue(queue)).id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(Active(SlotKey::SkillSpawn)),
			agent
				.get::<Track<Skill<Active>>>()
				.map(|t| t.value.data.clone()),
		);
	}

	#[test]
	fn no_dequeue_when_track_present() {
		let mut app = setup();
		let mut queue = Mock_Dequeue::default();
		queue.expect_dequeue().never().return_const(None);

		app.world.spawn((
			_TestQueue::Dequeue(queue),
			Track::new(Skill {
				data: Active(SlotKey::Hand(Side::Main)),
				..default()
			}),
		));

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
		queue.expect_dequeue().return_const(None);

		app.world.spawn(_TestQueue::Dequeue(queue));

		app.update();

		assert_eq!(None, app.world.get_resource::<FakeErrorLogManyResource>());
	}
}
