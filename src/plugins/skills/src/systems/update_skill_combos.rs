use crate::{
	components::slots::Slots,
	skills::{Queued, Skill},
	traits::{Flush, IsLingering, Iter, IterAddedMut, NextCombo},
};
use bevy::{
	ecs::{
		component::Component,
		system::{Query, Res},
	},
	time::Time,
};
use std::time::Duration;

type Components<'a, TCombos, TComboLinger, TSkills> = (
	&'a mut TCombos,
	Option<&'a mut TComboLinger>,
	&'a mut TSkills,
	&'a Slots,
);

pub(crate) fn update_skill_combos<
	TCombos: NextCombo + Flush + Component,
	TComboLinger: IsLingering + Flush + Component,
	TSkills: Iter<Skill<Queued>> + IterAddedMut<Skill<Queued>> + Component,
	TTime: Default + Sync + Send + 'static,
>(
	time: Res<Time<TTime>>,
	mut agents: Query<Components<TCombos, TComboLinger, TSkills>>,
) {
	let delta = time.delta();
	for (mut combos, mut linger, mut skills, slots) in &mut agents {
		let combos = combos.as_mut();
		let linger = linger.as_deref_mut();
		let skills = skills.as_mut();

		for skill in skills.iter_added_mut() {
			update_skill(combos, skill, slots);
		}

		for flushable in who_to_flush(combos, linger, skills, delta) {
			flushable.flush();
		}
	}
}

fn update_skill<TCombos: NextCombo>(
	combos: &mut TCombos,
	skill: &mut Skill<Queued>,
	slots: &Slots,
) {
	let Some(combo) = combos.next(&skill.data.slot_key, slots) else {
		return;
	};
	*skill = combo.with(skill.data.clone());
}

fn who_to_flush<
	'a,
	TCombos: Flush,
	TComboLinger: IsLingering + Flush,
	TSkills: Iter<Skill<Queued>>,
>(
	combos: &'a mut TCombos,
	linger: Option<&'a mut TComboLinger>,
	skills: &mut TSkills,
	delta: Duration,
) -> Vec<&'a mut dyn Flush> {
	if skills_queued(skills) {
		return one_or_empty(linger);
	}

	let Some(linger) = linger else {
		return vec![combos];
	};

	if !linger.is_lingering(delta) {
		return vec![combos, linger];
	}

	vec![]
}

fn skills_queued<TSkills: Iter<Skill<Queued>>>(skills: &mut TSkills) -> bool {
	skills.iter().next().is_some()
}

fn one_or_empty<TFlush: Flush>(linger: Option<&mut TFlush>) -> Vec<&mut dyn Flush> {
	linger.into_iter().map(as_dyn_flush).collect()
}

fn as_dyn_flush<TFlush: Flush>(value: &mut TFlush) -> &mut dyn Flush {
	value
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{Mounts, Slot},
		items::SlotKey,
		skills::{Queued, Skill},
		traits::IsLingering,
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
	};
	use mockall::{mock, predicate::eq};
	use std::{collections::HashMap, time::Duration};

	#[derive(Component, Default)]
	struct _Linger {
		mock: Mock_Linger,
	}

	impl Flush for _Linger {
		fn flush(&mut self) {
			self.mock.flush()
		}
	}

	impl IsLingering for _Linger {
		fn is_lingering(&mut self, delta: Duration) -> bool {
			self.mock.is_lingering(delta)
		}
	}

	mock! {
		_Linger {}
		impl Flush for _Linger {
			fn flush(&mut self);
		}
		impl IsLingering for _Linger {
			fn is_lingering(&mut self, delta: Duration) -> bool;
		}
	}

	#[derive(Component, Default)]
	struct _Combos {
		mock: Mock_Combos,
	}

	mock! {
		_Combos {}
		impl NextCombo for _Combos {
			fn next(&mut self, trigger: &SlotKey, slots: &Slots) -> Option<Skill> {}
		}
		impl Flush for _Combos {
			fn flush(&mut self) {}
		}
	}

	impl NextCombo for _Combos {
		fn next(&mut self, trigger: &SlotKey, slots: &Slots) -> Option<Skill> {
			self.mock.next(trigger, slots)
		}
	}

	impl Flush for _Combos {
		fn flush(&mut self) {
			self.mock.flush()
		}
	}

	#[derive(Component, Default, PartialEq, Debug)]
	struct _Skills {
		early: Vec<Skill<Queued>>,
		recent: Vec<Skill<Queued>>,
	}

	impl IterAddedMut<Skill<Queued>> for _Skills {
		fn iter_added_mut<'a>(
			&'a mut self,
		) -> impl DoubleEndedIterator<Item = &'a mut Skill<Queued>>
		where
			Skill<Queued>: 'a,
		{
			self.recent.iter_mut()
		}
	}

	impl Iter<Skill<Queued>> for _Skills {
		fn iter<'a>(&'a self) -> impl DoubleEndedIterator<Item = &'a Skill<Queued>>
		where
			Skill<Queued>: 'a,
		{
			self.early.iter().chain(self.recent.iter())
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
			update_skill_combos::<_Combos, _Linger, _Skills, Real>,
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
		let skill_a = Skill {
			name: "skill a",
			data: Queued {
				slot_key: SlotKey::Hand(Side::Main),
				..default()
			},
			..default()
		};
		let skill_b = Skill {
			name: "skill b",
			data: Queued {
				slot_key: SlotKey::Hand(Side::Off),
				..default()
			},
			..default()
		};
		let mut combos = _Combos::default();
		combos.mock.expect_flush().return_const(());
		combos
			.mock
			.expect_next()
			.times(1)
			.with(eq(SlotKey::Hand(Side::Main)), eq(slots.clone()))
			.return_const(Skill::default());
		combos
			.mock
			.expect_next()
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
		let skill_a = Skill {
			name: "skill a",
			data: Queued {
				slot_key: SlotKey::Hand(Side::Main),
				..default()
			},
			..default()
		};
		let skill_b = Skill {
			name: "skill b",
			data: Queued {
				slot_key: SlotKey::Hand(Side::Off),
				..default()
			},
			..default()
		};
		let mut combos = _Combos::default();
		combos.mock.expect_flush().return_const(());
		combos
			.mock
			.expect_next()
			.with(eq(SlotKey::Hand(Side::Main)), eq(slots.clone()))
			.return_const(Skill {
				name: "replace a",
				..default()
			});
		combos
			.mock
			.expect_next()
			.with(eq(SlotKey::Hand(Side::Off)), eq(slots.clone()))
			.return_const(Skill {
				name: "replace a",
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
					Skill {
						name: "replace a",
						data: Queued {
							slot_key: SlotKey::Hand(Side::Main),
							..default()
						},
						..default()
					},
					Skill {
						name: "replace a",
						data: Queued {
							slot_key: SlotKey::Hand(Side::Off),
							..default()
						},
						..default()
					}
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
			early: vec![Skill::default()],
			..default()
		};

		app.world.spawn((combos, skills, slots));
		app.update();
	}

	#[test]
	fn no_combo_flush_when_empty_and_linger_is_lingering() {
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
		let mut linger = _Linger::default();
		linger.mock.expect_is_lingering().return_const(true);
		linger.mock.expect_flush().return_const(());
		let skills = _Skills::default();

		app.world.spawn((combos, linger, skills, slots));
		app.update();
	}

	#[test]
	fn combo_flush_when_empty_and_linger_is_not_lingering() {
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
		let mut linger = _Linger::default();
		linger.mock.expect_is_lingering().return_const(false);
		linger.mock.expect_flush().return_const(());
		let skills = _Skills::default();

		app.world.spawn((combos, linger, skills, slots));
		app.update();
	}

	#[test]
	fn linger_flush_when_empty_and_linger_is_not_lingering() {
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
		let mut linger = _Linger::default();
		linger.mock.expect_is_lingering().return_const(false);
		linger.mock.expect_flush().times(1).return_const(());
		let skills = _Skills::default();

		app.world.spawn((combos, linger, skills, slots));
		app.update();
	}

	#[test]
	fn linger_flush_when_not_empty() {
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
		let mut linger = _Linger::default();
		linger.mock.expect_is_lingering().return_const(true);
		linger.mock.expect_flush().times(1).return_const(());
		let skills = _Skills {
			early: vec![Skill::default()],
			..default()
		};

		app.world.spawn((combos, linger, skills, slots));
		app.update();
	}

	#[test]
	fn no_linger_flush_when_empty_and_linger_is_lingering() {
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
		let mut linger = _Linger::default();
		linger.mock.expect_is_lingering().return_const(true);
		linger.mock.expect_flush().never().return_const(());
		let skills = _Skills::default();

		app.world.spawn((combos, linger, skills, slots));
		app.update();
	}

	#[test]
	fn do_not_test_for_linger_when_skill_queue_not_empty() {
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
		let mut linger = _Linger::default();
		linger
			.mock
			.expect_is_lingering()
			.never()
			.return_const(false);
		linger.mock.expect_flush().return_const(());
		let skills = _Skills {
			early: vec![Skill::default()],
			..default()
		};

		app.world.spawn((combos, linger, skills, slots));
		app.update();
	}

	#[test]
	fn call_is_lingering_with_delta() {
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
		let mut linger = _Linger::default();
		linger
			.mock
			.expect_is_lingering()
			.with(eq(Duration::from_secs(42)))
			.return_const(false);
		linger.mock.expect_flush().return_const(());
		let skills = _Skills::default();

		app.world.spawn((combos, linger, skills, slots));
		app.update();
	}
}
