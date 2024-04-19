use crate::{
	components::Slots,
	skill::{Queued, Skill},
	traits::{Flush, Iter, IterAddedMut, NextCombo},
};
use bevy::ecs::{component::Component, system::Query};

pub(crate) fn update_skill_combos<
	TCombos: NextCombo + Flush + Component,
	TSkills: Iter<Skill<Queued>> + IterAddedMut<Skill<Queued>> + Component,
>(
	mut agents: Query<(&mut TCombos, &mut TSkills, &Slots)>,
) {
	for (mut combos, mut skills, slots) in &mut agents {
		if skills.iter().next().is_none() {
			combos.flush();
		}
		for skill in skills.iter_added_mut() {
			let Some(combo) = combos.next(&skill.data.slot_key, slots) else {
				continue;
			};
			*skill = combo.with(skill.data.clone());
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{Slot, SlotKey, Slots},
		skill::{Queued, Skill},
	};
	use bevy::{
		app::{App, Update},
		ecs::{component::Component, entity::Entity},
		utils::default,
	};
	use common::{components::Side, test_tools::utils::SingleThreadedApp};
	use mockall::{mock, predicate::eq};
	use std::collections::HashMap;

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

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, update_skill_combos::<_Combos, _Skills>);

		app
	}

	#[test]
	fn call_next_with_new_skills() {
		let mut app = setup();
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Off),
			Slot {
				entity: Entity::from_raw(567),
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
				entity: Entity::from_raw(567),
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
	fn flush_when_empty() {
		let mut app = setup();
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Off),
			Slot {
				entity: Entity::from_raw(567),
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
	fn no_flush_when_not_empty() {
		let mut app = setup();
		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Off),
			Slot {
				entity: Entity::from_raw(567),
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
}
