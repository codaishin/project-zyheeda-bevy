use crate::{
	components::slots::Slots,
	skills::QueuedSkill,
	traits::{AdvanceCombo, Flush, IsTimedOut, IterAddedMut},
};
use bevy::prelude::*;
use common::traits::{iterate::Iterate, update_cumulative::CumulativeUpdate};
use std::time::Duration;

type Components<'a, TCombos, TComboTimeout, TSkills> = (
	&'a mut TCombos,
	Option<&'a mut TComboTimeout>,
	&'a mut TSkills,
	&'a Slots,
);

pub(crate) fn update_skill_combos<
	TCombos: AdvanceCombo + Flush + Component,
	TComboTimeout: IsTimedOut + CumulativeUpdate<Duration> + Flush + Component,
	TSkills: Iterate<QueuedSkill> + IterAddedMut<QueuedSkill> + Component,
	TTime: Default + Sync + Send + 'static,
>(
	time: Res<Time<TTime>>,
	mut agents: Query<Components<TCombos, TComboTimeout, TSkills>>,
) {
	let delta = time.delta();
	for (mut combos, timeout, mut skills, slots) in &mut agents {
		update_skills(&mut skills, &mut combos, slots);
		flush_combos_and_timeout(combos, timeout, skills, delta);
	}
}

fn update_skills<TSkills: IterAddedMut<QueuedSkill>, TCombos: AdvanceCombo>(
	skills: &mut Mut<TSkills>,
	combos: &mut Mut<TCombos>,
	slots: &Slots,
) {
	if skills.added_none() {
		return;
	}

	for skill in skills.iter_added_mut() {
		update_skill(combos, skill, slots);
	}
}

fn update_skill<TCombos: AdvanceCombo>(
	combos: &mut Mut<TCombos>,
	skill: &mut QueuedSkill,
	slots: &Slots,
) {
	let QueuedSkill {
		skill, slot_key, ..
	} = skill;

	let Some(combo_skill) = combos.advance(slot_key, slots) else {
		return;
	};

	*skill = combo_skill;
}

fn flush_combos_and_timeout<
	TCombos: Flush,
	TComboTimeout: CumulativeUpdate<Duration> + IsTimedOut + Flush,
	TSkills: Iterate<QueuedSkill>,
>(
	mut combos: Mut<TCombos>,
	timeout: Option<Mut<TComboTimeout>>,
	skills: Mut<TSkills>,
	delta: Duration,
) {
	match (skills_queued(skills), timeout) {
		(true, Some(mut timeout)) => timeout.flush(),
		(false, None) => combos.flush(),
		(false, Some(timeout)) => flush_when_timed_out(combos, timeout, delta),
		_ => {}
	}
}

fn skills_queued<TSkills: Iterate<QueuedSkill>>(skills: Mut<TSkills>) -> bool {
	skills.iterate().next().is_some()
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
	use crate::{
		items::slot_key::SlotKey,
		skills::{QueuedSkill, Skill},
		traits::IsTimedOut,
	};
	use bevy::{
		app::{App, Update},
		ecs::component::Component,
		time::{Real, Time},
		utils::default,
	};
	use common::{
		components::Side,
		test_tools::utils::{Changed, SingleThreadedApp, TickTime},
		traits::{nested_mock::NestedMocks, update_cumulative::CumulativeUpdate as UpdateTrait},
	};
	use macros::NestedMocks;
	use mockall::{mock, predicate::eq, Sequence};
	use std::{collections::HashMap, time::Duration};

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

	impl UpdateTrait<Duration> for _Timeout {
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
		impl UpdateTrait<Duration> for _Timeout {
			fn update_cumulative(&mut self, value: Duration);
		}
	}

	#[derive(Component, NestedMocks)]
	struct _Combos {
		mock: Mock_Combos,
	}

	mock! {
		_Combos {}
		impl AdvanceCombo for _Combos {
			fn advance(&mut self, trigger: &SlotKey, slots: &Slots) -> Option<Skill> {}
		}
		impl Flush for _Combos {
			fn flush(&mut self) {}
		}
	}

	impl AdvanceCombo for _Combos {
		fn advance(&mut self, trigger: &SlotKey, slots: &Slots) -> Option<Skill> {
			self.mock.advance(trigger, slots)
		}
	}

	impl Flush for _Combos {
		fn flush(&mut self) {
			self.mock.flush()
		}
	}

	#[derive(Component, Default, PartialEq, Debug)]
	struct _Skills {
		early: Vec<QueuedSkill>,
		recent: Vec<QueuedSkill>,
	}

	impl IterAddedMut<QueuedSkill> for _Skills {
		fn added_none(&self) -> bool {
			self.recent.is_empty()
		}

		fn iter_added_mut<'a>(&'a mut self) -> impl DoubleEndedIterator<Item = &'a mut QueuedSkill>
		where
			QueuedSkill: 'a,
		{
			self.recent.iter_mut()
		}
	}

	impl Iterate<QueuedSkill> for _Skills {
		fn iterate<'a>(&'a self) -> impl DoubleEndedIterator<Item = &'a QueuedSkill>
		where
			QueuedSkill: 'a,
		{
			self.early.iterate().chain(self.recent.iterate())
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<Time<Real>>();
		app.tick_time(Duration::ZERO);
		app.add_systems(
			Update,
			update_skill_combos::<_Combos, _Timeout, _Skills, Real>,
		);

		app
	}

	fn slots() -> Slots {
		Slots(HashMap::from([(SlotKey::BottomHand(Side::Left), None)]))
	}

	#[test]
	fn call_next_with_new_skills() {
		let mut app = setup();
		app.world_mut().spawn((
			_Combos::new().with_mock(|mock| {
				mock.expect_flush().return_const(());
				mock.expect_advance()
					.times(1)
					.with(eq(SlotKey::BottomHand(Side::Right)), eq(slots()))
					.return_const(Skill::default());
				mock.expect_advance()
					.times(1)
					.with(eq(SlotKey::BottomHand(Side::Left)), eq(slots()))
					.return_const(Skill::default());
			}),
			_Skills {
				recent: vec![
					QueuedSkill {
						skill: Skill {
							name: "skill a".to_owned(),
							..default()
						},
						slot_key: SlotKey::BottomHand(Side::Right),
						..default()
					},
					QueuedSkill {
						skill: Skill {
							name: "skill b".to_owned(),
							..default()
						},
						slot_key: SlotKey::BottomHand(Side::Left),
						..default()
					},
				],
				..default()
			},
			slots(),
		));

		app.update();
	}

	#[test]
	fn update_skill_with_combo_skills() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				_Combos::new().with_mock(|mock| {
					mock.expect_flush().return_const(());
					mock.expect_advance()
						.with(eq(SlotKey::BottomHand(Side::Right)), eq(slots()))
						.return_const(Skill {
							name: "replace a".to_owned(),
							..default()
						});
					mock.expect_advance()
						.with(eq(SlotKey::BottomHand(Side::Left)), eq(slots()))
						.return_const(Skill {
							name: "replace b".to_owned(),
							..default()
						});
				}),
				_Skills {
					recent: vec![
						QueuedSkill {
							skill: Skill {
								name: "skill a".to_owned(),
								..default()
							},
							slot_key: SlotKey::BottomHand(Side::Right),
							..default()
						},
						QueuedSkill {
							skill: Skill {
								name: "skill b".to_owned(),
								..default()
							},
							slot_key: SlotKey::BottomHand(Side::Left),
							..default()
						},
					],
					..default()
				},
				slots(),
			))
			.id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(
			Some(&_Skills {
				recent: vec![
					QueuedSkill {
						skill: Skill {
							name: "replace a".to_owned(),
							..default()
						},
						slot_key: SlotKey::BottomHand(Side::Right),
						..default()
					},
					QueuedSkill {
						skill: Skill {
							name: "replace b".to_owned(),
							..default()
						},
						slot_key: SlotKey::BottomHand(Side::Left),
						..default()
					},
				],
				..default()
			}),
			agent.get::<_Skills>()
		);
	}

	#[test]
	fn combo_flush_when_empty() {
		let mut app = setup();
		app.world_mut().spawn((
			_Combos::new().with_mock(|mock| {
				mock.expect_flush().times(1).return_const(());
			}),
			_Skills::default(),
			slots(),
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
			_Skills {
				early: vec![QueuedSkill::default()],
				..default()
			},
			slots(),
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
			_Skills::default(),
			slots(),
		));

		app.update();
	}

	#[test]
	fn combo_flush_when_empty_and_timed_out() {
		let mut app = setup();
		app.world_mut().spawn((
			_Combos::new().with_mock(|mock| {
				mock.expect_flush().times(1).return_const(());
			}),
			_Timeout::new().with_mock(|mock| {
				mock.expect_update_cumulative().return_const(());
				mock.expect_is_timed_out().return_const(true);
				mock.expect_flush().return_const(());
			}),
			_Skills::default(),
			slots(),
		));

		app.update();
	}

	#[test]
	fn timeout_flush_when_empty_and_is_timed_out() {
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
			_Skills::default(),
			slots(),
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
			_Skills {
				early: vec![QueuedSkill::default()],
				..default()
			},
			slots(),
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
			_Skills::default(),
			slots(),
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
			_Skills {
				early: vec![QueuedSkill::default()],
				..default()
			},
			slots(),
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
			_Skills::default(),
			slots(),
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
			_Skills::default(),
			slots(),
		));
		app.tick_time(Duration::from_secs(42));

		app.update();
	}

	#[test]
	fn skills_not_marked_changed_when_empty() {
		let mut app = setup().single_threaded(PostUpdate);
		let entity = app
			.world_mut()
			.spawn((
				Changed::<_Skills>::new(false),
				_Combos::new().with_mock(|mock| {
					mock.expect_flush().return_const(());
					mock.expect_advance().return_const(None);
				}),
				_Skills {
					recent: vec![],
					..default()
				},
				slots(),
			))
			.id();

		app.add_systems(PostUpdate, Changed::<_Skills>::detect);
		app.update(); // changed always true, because target was just added
		app.update();

		assert_eq!(
			Some(&Changed::new(false)),
			app.world().entity(entity).get::<Changed<_Skills>>(),
		)
	}

	#[test]
	fn combos_not_marked_changed_when_skills_not_empty_and_no_recently_added_skill() {
		let mut app = setup().single_threaded(PostUpdate);
		let entity = app
			.world_mut()
			.spawn((
				Changed::<_Combos>::new(false),
				_Combos::new().with_mock(|_| {}),
				_Skills {
					early: vec![QueuedSkill::default()],
					..default()
				},
				slots(),
			))
			.id();

		app.update();

		app.add_systems(PostUpdate, Changed::<_Combos>::detect);
		app.update(); // changed always true, because target was just added
		app.update();

		assert_eq!(
			Some(&false),
			app.world()
				.entity(entity)
				.get::<Changed<_Combos>>()
				.map(|Changed { changed, .. }| changed),
		)
	}
}
