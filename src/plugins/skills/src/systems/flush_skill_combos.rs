use crate::{
	skills::QueuedSkill,
	traits::{Flush, is_timed_out::IsTimedOut},
};
use bevy::{ecs::component::Mutable, prelude::*};
use common::traits::{iterate::Iterate, update_cumulative::CumulativeUpdate};
use std::time::Duration;

pub(crate) fn flush_skill_combos<TCombos, TComboTimeout, TTime, TQueue>(
	mut agents: Query<(&mut TCombos, Option<&mut TComboTimeout>, &TQueue)>,
	time: Res<Time<TTime>>,
) where
	TCombos: Flush + Component<Mutability = Mutable>,
	TComboTimeout:
		CumulativeUpdate<Duration> + IsTimedOut + Flush + Component<Mutability = Mutable>,
	TTime: Default + Sync + Send + 'static,
	for<'a> TQueue: Iterate<'a, TItem = &'a QueuedSkill> + Component + 'a,
{
	let delta = time.delta();

	for (mut combos, timeout, queue) in &mut agents {
		match (skills_queued(queue), timeout) {
			(true, Some(mut timeout)) => timeout.flush(),
			(false, None) => combos.flush(),
			(false, Some(timeout)) => flush_when_timed_out(combos, timeout, delta),
			_ => {}
		}
	}
}

fn skills_queued<TQueue>(queue: &TQueue) -> bool
where
	for<'a> TQueue: Iterate<'a, TItem = &'a QueuedSkill>,
{
	queue.iterate().next().is_some()
}

fn flush_when_timed_out<
	TComboTimeout: CumulativeUpdate<Duration> + IsTimedOut + Flush,
	TCombos: Flush,
>(
	mut combos: Mut<TCombos>,
	mut timeout: Mut<TComboTimeout>,
	delta: Duration,
) {
	timeout.update_cumulative(delta);
	if !timeout.is_timed_out() {
		return;
	}

	combos.flush();
	timeout.flush();
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::skills::QueuedSkill;
	use macros::NestedMocks;
	use mockall::{Sequence, mock, predicate::eq};
	use std::slice::Iter;
	use testing::{NestedMocks, SingleThreadedApp, TickTime};

	#[derive(Component, NestedMocks)]
	struct _Timeout {
		mock: Mock_Timeout,
	}

	impl Flush for _Timeout {
		fn flush(&mut self) {
			self.mock.flush()
		}
	}

	impl IsTimedOut for _Timeout {
		fn is_timed_out(&self) -> bool {
			self.mock.is_timed_out()
		}
	}

	impl CumulativeUpdate<Duration> for _Timeout {
		fn update_cumulative(&mut self, value: Duration) {
			self.mock.update_cumulative(value)
		}
	}

	mock! {
		_Timeout {}
		impl Flush for _Timeout {
			fn flush(&mut self);
		}
		impl IsTimedOut for _Timeout {
			fn is_timed_out(& self) -> bool;
		}
		impl CumulativeUpdate<Duration> for _Timeout {
			fn update_cumulative(&mut self, value: Duration);
		}
	}

	#[derive(Component, NestedMocks)]
	struct _Combos {
		mock: Mock_Combos,
	}

	mock! {
		_Combos {}
		impl Flush for _Combos {
			fn flush(&mut self) {}
		}
	}

	impl Flush for _Combos {
		fn flush(&mut self) {
			self.mock.flush()
		}
	}

	#[derive(Component, Default, PartialEq, Debug)]
	struct _Queue {
		skills: Vec<QueuedSkill>,
	}

	impl<'a> Iterate<'a> for _Queue {
		type TItem = &'a QueuedSkill;
		type TIter = Iter<'a, QueuedSkill>;

		fn iterate(&'a self) -> Self::TIter {
			self.skills.iter()
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<Time<Real>>();
		app.tick_time(Duration::ZERO);
		app.add_systems(
			Update,
			flush_skill_combos::<_Combos, _Timeout, Real, _Queue>,
		);

		app
	}

	#[test]
	fn combo_flush_when_empty() {
		let mut app = setup();
		app.world_mut().spawn((
			_Combos::new().with_mock(|mock| {
				mock.expect_flush().times(1).return_const(());
			}),
			_Queue::default(),
		));

		app.update();
	}

	#[test]
	fn no_combo_flush_when_not_empty() {
		let mut app = setup();
		app.world_mut().spawn((
			_Combos::new().with_mock(|mock| {
				mock.expect_flush().never().return_const(());
			}),
			_Queue {
				skills: vec![QueuedSkill::default()],
			},
		));

		app.update();
	}

	#[test]
	fn no_combo_flush_when_empty_and_not_timed_out() {
		let mut app = setup();
		app.world_mut().spawn((
			_Timeout::new().with_mock(|mock| {
				mock.expect_update_cumulative().return_const(());
				mock.expect_is_timed_out().return_const(false);
				mock.expect_flush().return_const(());
			}),
			_Combos::new().with_mock(|mock| {
				mock.expect_flush().never().return_const(());
			}),
			_Queue::default(),
		));

		app.update();
	}

	#[test]
	fn combo_flush_when_empty_and_timed_out() {
		let mut app = setup();
		app.world_mut().spawn((
			_Timeout::new().with_mock(|mock| {
				mock.expect_update_cumulative().return_const(());
				mock.expect_is_timed_out().return_const(true);
				mock.expect_flush().return_const(());
			}),
			_Combos::new().with_mock(|mock| {
				mock.expect_flush().times(1).return_const(());
			}),
			_Queue::default(),
		));

		app.update();
	}

	#[test]
	fn timeout_flush_when_empty_and_timed_out() {
		let mut app = setup();
		app.world_mut().spawn((
			_Combos::new().with_mock(|mock| {
				mock.expect_flush().return_const(());
			}),
			_Timeout::new().with_mock(|mock| {
				mock.expect_update_cumulative().return_const(());
				mock.expect_is_timed_out().return_const(true);
				mock.expect_flush().times(1).return_const(());
			}),
			_Queue::default(),
		));

		app.update();
	}

	#[test]
	fn timeout_flush_when_not_empty() {
		let mut app = setup();
		app.world_mut().spawn((
			_Combos::new().with_mock(|mock| {
				mock.expect_flush().return_const(());
			}),
			_Timeout::new().with_mock(|mock| {
				mock.expect_update_cumulative().return_const(());
				mock.expect_is_timed_out().return_const(false);
				mock.expect_flush().times(1).return_const(());
			}),
			_Queue {
				skills: vec![QueuedSkill::default()],
			},
		));

		app.update();
	}

	#[test]
	fn no_timeout_flush_when_empty_and_is_not_timed_out() {
		let mut app = setup();
		app.world_mut().spawn((
			_Combos::new().with_mock(|mock| {
				mock.expect_flush().return_const(());
			}),
			_Timeout::new().with_mock(|mock| {
				mock.expect_update_cumulative().return_const(());
				mock.expect_is_timed_out().return_const(false);
				mock.expect_flush().never().return_const(());
			}),
			_Queue::default(),
		));

		app.update();
	}

	#[test]
	fn do_not_test_for_timeout_when_skill_queue_not_empty() {
		let mut app = setup();
		app.world_mut().spawn((
			_Combos::new().with_mock(|mock| {
				mock.expect_flush().return_const(());
			}),
			_Timeout::new().with_mock(|mock| {
				mock.expect_update_cumulative().return_const(());
				mock.expect_is_timed_out().never().return_const(false);
				mock.expect_flush().return_const(());
			}),
			_Queue {
				skills: vec![QueuedSkill::default()],
			},
		));

		app.update();
	}

	#[test]
	fn call_is_timeout_with_delta() {
		let mut app = setup();
		app.tick_time(Duration::from_secs(42));
		app.world_mut().spawn((
			_Combos::new().with_mock(|mock| {
				mock.expect_flush().return_const(());
			}),
			_Timeout::new().with_mock(|mock| {
				mock.expect_update_cumulative()
					.with(eq(Duration::from_secs(42)))
					.return_const(());
				mock.expect_is_timed_out().return_const(false);
				mock.expect_flush().return_const(());
			}),
			_Queue::default(),
		));

		app.update();
	}

	#[test]
	fn call_update_and_timeout_in_sequence() {
		let mut app = setup();
		app.world_mut().spawn((
			_Combos::new().with_mock(|mock| {
				mock.expect_flush().return_const(());
			}),
			_Timeout::new().with_mock(|mock| {
				let mut seq = Sequence::default();
				mock.expect_update_cumulative()
					.times(1)
					.in_sequence(&mut seq)
					.return_const(());
				mock.expect_is_timed_out()
					.times(1)
					.in_sequence(&mut seq)
					.return_const(false);
				mock.expect_flush().return_const(());
			}),
			_Queue::default(),
		));
		app.tick_time(Duration::from_secs(42));

		app.update();
	}
}
