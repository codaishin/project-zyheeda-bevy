use crate::components::quickbar_panel::QuickbarPanel;
use bevy::prelude::{Component, Entity, Query, With};
use common::{
	components::Player,
	traits::{iterate::Iterate, load_asset::Path},
};
use skills::{
	components::slots::Slots,
	skills::{Queued, Skill},
	traits::{IsTimedOut, PeekNext},
};

type IconPath = Option<fn() -> Path>;

type PlayerComponents<'a, TQueue, TCombos, TComboTimeout> = (
	&'a Slots,
	&'a TQueue,
	Option<&'a TCombos>,
	Option<&'a TComboTimeout>,
);

pub(crate) fn get_quickbar_icons<TQueue, TCombos, TComboTimeout>(
	players: Query<PlayerComponents<TQueue, TCombos, TComboTimeout>, With<Player>>,
	panels: Query<(Entity, &mut QuickbarPanel)>,
) -> Vec<(Entity, Option<Path>)>
where
	TQueue: Component + Iterate<Skill<Queued>>,
	TCombos: Component + PeekNext<Skill>,
	TComboTimeout: Component + IsTimedOut,
{
	let Ok((slots, queue, combos, combo_timeout)) = players.get_single() else {
		return vec![];
	};
	let get_icon_path = |(entity, panel): (Entity, &QuickbarPanel)| {
		let icon = if_active_skill_icon(panel, queue)
			.or_else(if_combo_skill_icon(panel, slots, combos, combo_timeout))
			.or_else(if_item_skill_icon(panel, slots));
		let icon = icon.flatten().map(|resolve_path| resolve_path());

		(entity, icon)
	};

	panels.iter().map(get_icon_path).collect()
}

fn if_active_skill_icon<TQueue: Iterate<Skill<Queued>>>(
	panel: &QuickbarPanel,
	queue: &TQueue,
) -> Option<IconPath> {
	let active_skill = queue.iterate().next()?;

	if active_skill.data.slot_key != panel.key {
		return None;
	}

	Some(active_skill.icon)
}

fn if_combo_skill_icon<'a, TCombos: PeekNext<Skill>, TComboTimeout: IsTimedOut>(
	panel: &'a QuickbarPanel,
	slots: &'a Slots,
	combos: Option<&'a TCombos>,
	timed_out: Option<&'a TComboTimeout>,
) -> impl FnOnce() -> Option<IconPath> + 'a {
	move || {
		if timed_out?.is_timed_out() {
			return None;
		}
		let next_combo = combos?.peek_next(&panel.key, slots)?;
		Some(next_combo.icon)
	}
}

fn if_item_skill_icon<'a>(
	panel: &'a QuickbarPanel,
	slots: &'a Slots,
) -> impl FnOnce() -> Option<IconPath> + 'a {
	|| {
		let slot = slots.0.get(&panel.key)?;
		let item = slot.item.as_ref()?;
		let skill = item.skill.as_ref()?;

		Some(skill.icon)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{components::quickbar_panel::QuickbarPanel, tools::PanelState};
	use bevy::{
		app::{App, Update},
		ecs::system::In,
		prelude::{default, Commands, IntoSystem, Resource},
	};
	use common::{
		components::{Player, Side},
		test_tools::utils::SingleThreadedApp,
	};
	use mockall::{automock, predicate::eq};
	use skills::{
		components::{slots::Slots, Mounts, Slot},
		items::{slot_key::SlotKey, Item},
		skills::Activation,
	};
	use std::collections::HashMap;

	#[derive(Component, Default)]
	struct _Queue(Vec<Skill<Queued>>);

	impl Iterate<Skill<Queued>> for _Queue {
		fn iterate<'a>(&'a self) -> impl DoubleEndedIterator<Item = &'a Skill<Queued>>
		where
			Skill<Queued>: 'a,
		{
			self.0.iter()
		}
	}

	#[derive(Component, Default)]
	struct _Combos {
		mock: Mock_Combos,
	}

	#[automock]
	impl PeekNext<Skill> for _Combos {
		fn peek_next(&self, trigger: &SlotKey, slots: &Slots) -> Option<Skill> {
			self.mock.peek_next(trigger, slots)
		}
	}

	#[derive(Component)]
	struct _ComboTimeout(bool);

	impl IsTimedOut for _ComboTimeout {
		fn is_timed_out(&self) -> bool {
			self.0
		}
	}

	#[derive(Resource, Default)]
	struct _Result(Vec<(Entity, Option<Path>)>);

	fn store_result(result: In<Vec<(Entity, Option<Path>)>>, mut commands: Commands) {
		commands.insert_resource(_Result(result.0));
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<_Result>();
		app.add_systems(
			Update,
			get_quickbar_icons::<_Queue, _Combos, _ComboTimeout>.pipe(store_result),
		);

		app
	}

	fn arbitrary_mounts() -> Mounts<Entity> {
		Mounts {
			hand: Entity::from_raw(100),
			forearm: Entity::from_raw(200),
		}
	}

	#[test]
	fn return_combo_skill_icon_when_no_skill_active_and_combo_not_timed_out() {
		let mut app = setup();
		let mut combos = _Combos::default();

		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				mounts: arbitrary_mounts(),
				item: Some(Item {
					skill: Some(Skill {
						icon: Some(|| Path::from("item_skill/icon/path")),
						..default()
					}),
					..default()
				}),
			},
		)]));
		combos.mock.expect_peek_next().return_const(Skill {
			icon: Some(|| Path::from("combo_skill/icon/path")),
			..default()
		});
		app.world.spawn((
			Player,
			slots,
			_Queue::default(),
			combos,
			_ComboTimeout(false),
		));
		let panel = app
			.world
			.spawn(QuickbarPanel {
				key: SlotKey::Hand(Side::Main),
				state: PanelState::Empty,
			})
			.id();

		app.update();

		let result = app.world.resource::<_Result>();
		assert_eq!(
			vec![(panel, Some(Path::from("combo_skill/icon/path")))],
			result.0
		);
	}

	#[test]
	fn peek_combo_with_correct_arguments() {
		let mut app = setup();
		let mut combos = _Combos::default();

		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				mounts: arbitrary_mounts(),
				item: Some(Item {
					skill: Some(Skill {
						icon: Some(|| Path::from("item_skill/icon/path")),
						..default()
					}),
					..default()
				}),
			},
		)]));
		combos
			.mock
			.expect_peek_next()
			.times(1)
			.with(eq(SlotKey::Hand(Side::Off)), eq(slots.clone()))
			.return_const(None);
		app.world.spawn((
			Player,
			slots,
			_Queue::default(),
			combos,
			_ComboTimeout(false),
		));
		app.world.spawn(QuickbarPanel {
			key: SlotKey::Hand(Side::Off),
			state: PanelState::Empty,
		});

		app.update();
	}

	#[test]
	fn return_item_skill_icon_when_no_skill_active_and_combo_timed_out() {
		let mut app = setup();
		let mut combos = _Combos::default();

		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				mounts: arbitrary_mounts(),
				item: Some(Item {
					skill: Some(Skill {
						icon: Some(|| Path::from("item_skill/icon/path")),
						..default()
					}),
					..default()
				}),
			},
		)]));
		combos.mock.expect_peek_next().return_const(Skill {
			icon: Some(|| Path::from("combo_skill/icon/path")),
			..default()
		});
		app.world.spawn((
			Player,
			slots,
			_Queue::default(),
			combos,
			_ComboTimeout(true),
		));
		let panel = app
			.world
			.spawn(QuickbarPanel {
				key: SlotKey::Hand(Side::Main),
				state: PanelState::Empty,
			})
			.id();

		app.update();

		let result = app.world.resource::<_Result>();
		assert_eq!(
			vec![(panel, Some(Path::from("item_skill/icon/path")))],
			result.0
		);
	}

	#[test]
	fn return_item_skill_icon_when_no_skill_active_and_combo_empty_but_not_timed_out() {
		let mut app = setup();
		let mut combos = _Combos::default();

		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				mounts: arbitrary_mounts(),
				item: Some(Item {
					skill: Some(Skill {
						icon: Some(|| Path::from("item_skill/icon/path")),
						..default()
					}),
					..default()
				}),
			},
		)]));
		combos.mock.expect_peek_next().return_const(None);
		app.world.spawn((
			Player,
			slots,
			_Queue::default(),
			combos,
			_ComboTimeout(false),
		));
		let panel = app
			.world
			.spawn(QuickbarPanel {
				key: SlotKey::Hand(Side::Main),
				state: PanelState::Empty,
			})
			.id();

		app.update();

		let result = app.world.resource::<_Result>();
		assert_eq!(
			vec![(panel, Some(Path::from("item_skill/icon/path")))],
			result.0
		);
	}

	#[test]
	fn return_active_skill_icon_when_skill_active() {
		let mut app = setup();
		let mut combos = _Combos::default();

		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				mounts: arbitrary_mounts(),
				item: Some(Item {
					skill: Some(Skill {
						icon: Some(|| Path::from("item_skill/icon/path")),
						..default()
					}),
					..default()
				}),
			},
		)]));
		combos.mock.expect_peek_next().return_const(Skill {
			icon: Some(|| Path::from("combo_skill/icon/path")),
			..default()
		});
		app.world.spawn((
			Player,
			slots,
			_Queue(vec![Skill {
				icon: Some(|| Path::from("active_skill/icon/path")),
				data: Queued {
					slot_key: SlotKey::Hand(Side::Off),
					mode: Activation::Waiting,
				},
				..default()
			}]),
			combos,
			_ComboTimeout(true),
		));
		let panel = app
			.world
			.spawn(QuickbarPanel {
				key: SlotKey::Hand(Side::Off),
				state: PanelState::Empty,
			})
			.id();

		app.update();

		let result = app.world.resource::<_Result>();
		assert_eq!(
			vec![(panel, Some(Path::from("active_skill/icon/path")))],
			result.0
		);
	}

	#[test]
	fn return_item_skill_icon_when_skill_active_for_other_slot() {
		let mut app = setup();
		let mut combos = _Combos::default();

		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				mounts: arbitrary_mounts(),
				item: Some(Item {
					skill: Some(Skill {
						icon: Some(|| Path::from("item_skill/icon/path")),
						..default()
					}),
					..default()
				}),
			},
		)]));
		combos.mock.expect_peek_next().return_const(Skill {
			icon: Some(|| Path::from("combo_skill/icon/path")),
			..default()
		});
		app.world.spawn((
			Player,
			slots,
			_Queue(vec![Skill {
				icon: Some(|| Path::from("active_skill/icon/path")),
				data: Queued {
					slot_key: SlotKey::Hand(Side::Off),
					mode: Activation::Waiting,
				},
				..default()
			}]),
			combos,
			_ComboTimeout(true),
		));
		let panel = app
			.world
			.spawn(QuickbarPanel {
				key: SlotKey::Hand(Side::Main),
				state: PanelState::Empty,
			})
			.id();

		app.update();

		let result = app.world.resource::<_Result>();
		assert_eq!(
			vec![(panel, Some(Path::from("item_skill/icon/path")))],
			result.0
		);
	}

	#[test]
	fn return_item_skill_icon_when_no_skill_active_and_no_combo_components_present() {
		let mut app = setup();

		let slots = Slots(HashMap::from([(
			SlotKey::Hand(Side::Main),
			Slot {
				mounts: arbitrary_mounts(),
				item: Some(Item {
					skill: Some(Skill {
						icon: Some(|| Path::from("item_skill/icon/path")),
						..default()
					}),
					..default()
				}),
			},
		)]));
		app.world.spawn((Player, slots, _Queue::default()));
		let panel = app
			.world
			.spawn(QuickbarPanel {
				key: SlotKey::Hand(Side::Main),
				state: PanelState::Empty,
			})
			.id();

		app.update();

		let result = app.world.resource::<_Result>();
		assert_eq!(
			vec![(panel, Some(Path::from("item_skill/icon/path")))],
			result.0
		);
	}
}
