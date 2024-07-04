use crate::{
	components::slots::Slots,
	skills::QueuedSkill,
	traits::{AdvanceCombo, Flush, IsTimedOut, IterAddedMut},
};
use bevy::{
	ecs::{
		component::Component,
		system::{Query, Res},
	},
	time::Time,
};
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
	for (mut combos, mut timeout, mut skills, slots) in &mut agents {
		let combos = combos.as_mut();
		let timeout = timeout.as_deref_mut();
		let skills = skills.as_mut();

		for skill in skills.iter_added_mut() {
			update_skill(combos, skill, slots);
		}

		for flushable in who_to_flush(combos, timeout, skills, delta) {
			flushable.flush();
		}
	}
}

fn update_skill<TCombos: AdvanceCombo>(
	combos: &mut TCombos,
	skill: &mut QueuedSkill,
	slots: &Slots,
) {
	let Some(combo_skill) = combos.advance(&skill.slot_key, slots) else {
		return;
	};
	*skill = QueuedSkill {
		skill: combo_skill,
		slot_key: skill.slot_key,
		mode: skill.mode.clone(),
	};
}

fn who_to_flush<
	'a,
	TCombos: Flush,
	TComboTimeout: CumulativeUpdate<Duration> + IsTimedOut + Flush,
	TSkills: Iterate<QueuedSkill>,
>(
	combos: &'a mut TCombos,
	timeout: Option<&'a mut TComboTimeout>,
	skills: &mut TSkills,
	delta: Duration,
) -> Vec<&'a mut dyn Flush> {
	if skills_queued(skills) {
		return one_or_empty(timeout);
	}

	let Some(timeout) = timeout else {
		return vec![combos];
	};

	timeout.update_cumulative(delta);
	if timeout.is_timed_out() {
		return vec![combos, timeout];
	}

	vec![]
}

fn skills_queued<TSkills: Iterate<QueuedSkill>>(skills: &mut TSkills) -> bool {
	skills.iterate().next().is_some()
}

fn one_or_empty<TFlush: Flush>(flush: Option<&mut TFlush>) -> Vec<&mut dyn Flush> {
	flush.into_iter().map(as_dyn_flush).collect()
}

fn as_dyn_flush<TFlush: Flush>(value: &mut TFlush) -> &mut dyn Flush {
	value
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{Mounts, Slot},
		items::slot_key::SlotKey,
		skills::{QueuedSkill, Skill},
		traits::IsTimedOut,
	};
	use bevy::{
		app::{App, Update},
		ecs::{component::Component, entity::Entity},
		time::{Real, Time},
		utils::default,
	};
	use common::{
		components::Side,
		test_tools::utils::{SingleThreadedApp, TickTime},
		traits::update_cumulative::CumulativeUpdate as UpdateTrait,
	};
	use mockall::{mock, predicate::eq, Sequence};
	use std::{collections::HashMap, time::Duration};

	#[derive(Component, Default)]
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

	#[derive(Component, Default)]
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

	fn mounts() -> Mounts<Entity> {
		Mounts {
			hand: Entity::from_raw(100),
			forearm: Entity::from_raw(200),
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

	#[test]
	fn call_next_with_new_skills() {
		let mut app = setup();
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Off),
			Slot {
				mounts: mounts(),
				item: None,
			},
		)]));
		let skill_a = QueuedSkill {
			skill: Skill {
				name: "skill a".to_owned(),
				..default()
			},
			slot_key: SlotKey::Hand(Side::Main),
			..default()
		};
		let skill_b = QueuedSkill {
			skill: Skill {
				name: "skill b".to_owned(),
				..default()
			},
			slot_key: SlotKey::Hand(Side::Off),
			..default()
		};
		let mut combos = _Combos::default();
		combos.mock.expect_flush().return_const(());
		combos
			.mock
			.expect_advance()
			.times(1)
			.with(eq(SlotKey::Hand(Side::Main)), eq(slots.clone()))
			.return_const(Skill::default());
		combos
			.mock
			.expect_advance()
			.times(1)
			.with(eq(SlotKey::Hand(Side::Off)), eq(slots.clone()))
			.return_const(Skill::default());
		let skills = _Skills {
			recent: vec![skill_a, skill_b],
			..default()
		};

		app.world.spawn((combos, skills, slots));
		app.update();
	}

	#[test]
	fn update_skill_with_combo_skills() {
		let mut app = setup();
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Off),
			Slot {
				mounts: mounts(),
				item: None,
			},
		)]));
		let skill_a = QueuedSkill {
			skill: Skill {
				name: "skill a".to_owned(),
				..default()
			},
			slot_key: SlotKey::Hand(Side::Main),
			..default()
		};
		let skill_b = QueuedSkill {
			skill: Skill {
				name: "skill b".to_owned(),
				..default()
			},
			slot_key: SlotKey::Hand(Side::Off),
			..default()
		};
		let mut combos = _Combos::default();
		combos.mock.expect_flush().return_const(());
		combos
			.mock
			.expect_advance()
			.with(eq(SlotKey::Hand(Side::Main)), eq(slots.clone()))
			.return_const(Skill {
				name: "replace a".to_owned(),
				..default()
			});
		combos
			.mock
			.expect_advance()
			.with(eq(SlotKey::Hand(Side::Off)), eq(slots.clone()))
			.return_const(Skill {
				name: "replace b".to_owned(),
				..default()
			});
		let skills = _Skills {
			recent: vec![skill_a, skill_b],
			..default()
		};

		let agent = app.world.spawn((combos, skills, slots)).id();
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&_Skills {
				recent: vec![
					QueuedSkill {
						skill: Skill {
							name: "replace a".to_owned(),
							..default()
						},
						slot_key: SlotKey::Hand(Side::Main),
						..default()
					},
					QueuedSkill {
						skill: Skill {
							name: "replace b".to_owned(),
							..default()
						},
						slot_key: SlotKey::Hand(Side::Off),
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
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Off),
			Slot {
				mounts: mounts(),
				item: None,
			},
		)]));
		let mut combos = _Combos::default();
		combos.mock.expect_flush().times(1).return_const(());
		let skills = _Skills::default();

		app.world.spawn((combos, skills, slots));
		app.update();
	}

	#[test]
	fn no_combo_flush_when_not_empty() {
		let mut app = setup();
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Off),
			Slot {
				mounts: mounts(),
				item: None,
			},
		)]));
		let mut combos = _Combos::default();
		combos.mock.expect_flush().never().return_const(());
		let skills = _Skills {
			early: vec![QueuedSkill::default()],
			..default()
		};

		app.world.spawn((combos, skills, slots));
		app.update();
	}

	#[test]
	fn no_combo_flush_when_empty_and_not_timed_out() {
		let mut app = setup();
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Off),
			Slot {
				mounts: mounts(),
				item: None,
			},
		)]));
		let mut combos = _Combos::default();
		combos.mock.expect_flush().never().return_const(());
		let mut timeout = _Timeout::default();
		timeout.mock.expect_update_cumulative().return_const(());
		timeout.mock.expect_is_timed_out().return_const(false);
		timeout.mock.expect_flush().return_const(());
		let skills = _Skills::default();

		app.world.spawn((combos, timeout, skills, slots));
		app.update();
	}

	#[test]
	fn combo_flush_when_empty_and_timed_out() {
		let mut app = setup();
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Off),
			Slot {
				mounts: mounts(),
				item: None,
			},
		)]));
		let mut combos = _Combos::default();
		combos.mock.expect_flush().times(1).return_const(());
		let mut timeout = _Timeout::default();
		timeout.mock.expect_update_cumulative().return_const(());
		timeout.mock.expect_is_timed_out().return_const(true);
		timeout.mock.expect_flush().return_const(());
		let skills = _Skills::default();

		app.world.spawn((combos, timeout, skills, slots));
		app.update();
	}

	#[test]
	fn timeout_flush_when_empty_and_is_timed_out() {
		let mut app = setup();
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Off),
			Slot {
				mounts: mounts(),
				item: None,
			},
		)]));
		let mut combos = _Combos::default();
		combos.mock.expect_flush().return_const(());
		let mut timeout = _Timeout::default();
		timeout.mock.expect_update_cumulative().return_const(());
		timeout.mock.expect_is_timed_out().return_const(true);
		timeout.mock.expect_flush().times(1).return_const(());
		let skills = _Skills::default();

		app.world.spawn((combos, timeout, skills, slots));
		app.update();
	}

	#[test]
	fn timeout_flush_when_not_empty() {
		let mut app = setup();
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Off),
			Slot {
				mounts: mounts(),
				item: None,
			},
		)]));
		let mut combos = _Combos::default();
		combos.mock.expect_flush().return_const(());
		let mut timeout = _Timeout::default();
		timeout.mock.expect_update_cumulative().return_const(());
		timeout.mock.expect_is_timed_out().return_const(false);
		timeout.mock.expect_flush().times(1).return_const(());
		let skills = _Skills {
			early: vec![QueuedSkill::default()],
			..default()
		};

		app.world.spawn((combos, timeout, skills, slots));
		app.update();
	}

	#[test]
	fn no_timeout_flush_when_empty_and_is_not_timed_out() {
		let mut app = setup();
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Off),
			Slot {
				mounts: mounts(),
				item: None,
			},
		)]));
		let mut combos = _Combos::default();
		combos.mock.expect_flush().return_const(());
		let mut timeout = _Timeout::default();
		timeout.mock.expect_update_cumulative().return_const(());
		timeout.mock.expect_is_timed_out().return_const(false);
		timeout.mock.expect_flush().never().return_const(());
		let skills = _Skills::default();

		app.world.spawn((combos, timeout, skills, slots));
		app.update();
	}

	#[test]
	fn do_not_test_for_timeout_when_skill_queue_not_empty() {
		let mut app = setup();
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Off),
			Slot {
				mounts: mounts(),
				item: None,
			},
		)]));
		let mut combos = _Combos::default();
		combos.mock.expect_flush().return_const(());
		let mut timeout = _Timeout::default();
		timeout.mock.expect_update_cumulative().return_const(());
		timeout
			.mock
			.expect_is_timed_out()
			.never()
			.return_const(false);
		timeout.mock.expect_flush().return_const(());
		let skills = _Skills {
			early: vec![QueuedSkill::default()],
			..default()
		};

		app.world.spawn((combos, timeout, skills, slots));
		app.update();
	}

	#[test]
	fn call_is_timeout_with_delta() {
		let mut app = setup();
		app.tick_time(Duration::from_secs(42));
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Off),
			Slot {
				mounts: mounts(),
				item: None,
			},
		)]));
		let mut combos = _Combos::default();
		combos.mock.expect_flush().return_const(());
		let mut timeout = _Timeout::default();
		timeout
			.mock
			.expect_update_cumulative()
			.with(eq(Duration::from_secs(42)))
			.return_const(());
		timeout.mock.expect_is_timed_out().return_const(false);
		timeout.mock.expect_flush().return_const(());
		let skills = _Skills::default();

		app.world.spawn((combos, timeout, skills, slots));
		app.update();
	}

	#[test]
	fn call_update_and_timeout_in_sequence() {
		let mut app = setup();
		app.tick_time(Duration::from_secs(42));
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Off),
			Slot {
				mounts: mounts(),
				item: None,
			},
		)]));
		let mut combos = _Combos::default();
		combos.mock.expect_flush().return_const(());
		let mut seq = Sequence::default();
		let mut timeout = _Timeout::default();
		timeout
			.mock
			.expect_update_cumulative()
			.times(1)
			.in_sequence(&mut seq)
			.return_const(());
		timeout
			.mock
			.expect_is_timed_out()
			.times(1)
			.in_sequence(&mut seq)
			.return_const(false);
		timeout.mock.expect_flush().return_const(());
		let skills = _Skills::default();

		app.world.spawn((combos, timeout, skills, slots));
		app.update();
	}
}
