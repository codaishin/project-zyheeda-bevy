use crate::components::quickbar_panel::QuickbarPanel;
use bevy::prelude::*;
use common::traits::{accessors::get::GetRef, iterate::Iterate};
use skills::{
	components::slots::Slots,
	item::Item,
	skills::{QueuedSkill, Skill},
	traits::{IsTimedOut, PeekNext},
};

type PlayerComponents<'a, TQueue, TCombos, TComboTimeout> = (
	&'a Slots,
	&'a TQueue,
	Option<&'a TCombos>,
	Option<&'a TComboTimeout>,
);

pub(crate) fn get_quickbar_icons<TPlayer, TQueue, TCombos, TComboTimeout>(
	players: Query<PlayerComponents<TQueue, TCombos, TComboTimeout>, With<TPlayer>>,
	panels: Query<(Entity, &mut QuickbarPanel)>,
	items: Res<Assets<Item>>,
	skills: Res<Assets<Skill>>,
) -> Vec<(Entity, Option<Handle<Image>>)>
where
	TPlayer: Component,
	TQueue: Component + Iterate<QueuedSkill>,
	TCombos: Component + PeekNext<Skill>,
	TComboTimeout: Component + IsTimedOut,
{
	let Ok((slots, queue, combos, timeout)) = players.get_single() else {
		return vec![];
	};
	let get_icon_path = |(entity, panel): (Entity, &QuickbarPanel)| {
		let icon = active_skill_icon(panel, queue)
			.or_else(combo_skill_icon(panel, &items, slots, combos, timeout))
			.or_else(item_skill_icon(panel, &items, &skills, slots));

		(entity, icon)
	};

	panels.iter().map(get_icon_path).collect()
}

fn active_skill_icon<TQueue: Iterate<QueuedSkill>>(
	panel: &QuickbarPanel,
	queue: &TQueue,
) -> Option<Handle<Image>> {
	let active_skill = queue.iterate().next()?;

	if active_skill.slot_key != panel.key {
		return None;
	}

	active_skill.skill.icon.clone()
}

fn combo_skill_icon<'a, TCombos: PeekNext<Skill>, TComboTimeout: IsTimedOut>(
	panel: &'a QuickbarPanel,
	items: &'a Assets<Item>,
	slots: &'a Slots,
	combos: Option<&'a TCombos>,
	timed_out: Option<&'a TComboTimeout>,
) -> impl FnOnce() -> Option<Handle<Image>> + 'a {
	move || {
		if timed_out?.is_timed_out() {
			return None;
		}
		let item_handle = slots.get(&panel.key)?;
		let item = items.get(item_handle.id())?;
		let next_combo = combos?.peek_next(&panel.key, &item.item_type)?;
		next_combo.icon
	}
}

fn item_skill_icon<'a>(
	panel: &'a QuickbarPanel,
	items: &'a Assets<Item>,
	skills: &'a Assets<Skill>,
	slots: &'a Slots,
) -> impl FnOnce() -> Option<Handle<Image>> + 'a {
	|| {
		slots
			.get(&panel.key)
			.and_then(|item| items.get(item))
			.and_then(|item| item.skill.as_ref())
			.and_then(|skill| skills.get(skill))
			.and_then(|skill| skill.icon.clone())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{components::quickbar_panel::QuickbarPanel, tools::PanelState};
	use common::{
		test_tools::utils::SingleThreadedApp,
		tools::slot_key::{Side, SlotKey},
		traits::nested_mock::NestedMocks,
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use skills::{
		components::slots::Slots,
		item::{item_type::SkillItemType, Item},
		skills::Activation,
	};
	use std::collections::HashMap;
	use uuid::Uuid;

	#[derive(Component)]
	struct _Player;

	#[derive(Component, Default)]
	struct _Queue(Vec<QueuedSkill>);

	impl Iterate<QueuedSkill> for _Queue {
		fn iterate<'a>(&'a self) -> impl DoubleEndedIterator<Item = &'a QueuedSkill>
		where
			QueuedSkill: 'a,
		{
			self.0.iter()
		}
	}

	#[derive(Component, NestedMocks)]
	struct _Combos {
		mock: Mock_Combos,
	}

	#[automock]
	impl PeekNext<Skill> for _Combos {
		fn peek_next(&self, trigger: &SlotKey, item_type: &SkillItemType) -> Option<Skill> {
			self.mock.peek_next(trigger, item_type)
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
	struct _Result(Vec<(Entity, Option<Handle<Image>>)>);

	fn store_result(result: In<Vec<(Entity, Option<Handle<Image>>)>>, mut commands: Commands) {
		commands.insert_resource(_Result(result.0));
	}

	fn setup(items: Assets<Item>, skills: Assets<Skill>) -> App {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(items);
		app.insert_resource(skills);
		app.init_resource::<_Result>();
		app.add_systems(
			Update,
			get_quickbar_icons::<_Player, _Queue, _Combos, _ComboTimeout>.pipe(store_result),
		);

		app
	}

	fn get_handle<TAsset: Asset>(name: &str) -> Handle<TAsset> {
		match name {
			"item skill" => Handle::Weak(AssetId::Uuid {
				uuid: Uuid::from_u128(0x5e65c0ee_c118_4fa7_a888_6a988f139c1e),
			}),
			"combo skill" => Handle::Weak(AssetId::Uuid {
				uuid: Uuid::from_u128(0x7647d77f_2826_4baf_b5b9_195524f1c975),
			}),
			"active skill" => Handle::Weak(AssetId::Uuid {
				uuid: Uuid::from_u128(0x70bf5ce3_df23_40aa_80c9_51b2c5baa23c),
			}),
			_ => Handle::default(),
		}
	}

	#[test]
	fn return_combo_skill_icon_when_no_skill_active_and_combo_not_timed_out() {
		let (slots, items, skills) = setup_slots([(
			SlotKey::BottomHand(Side::Right),
			SkillItemType::Pistol,
			Skill {
				icon: Some(get_handle("item skill")),
				..default()
			},
		)]);
		let mut app = setup(items, skills);
		app.world_mut().spawn((
			_Player,
			slots,
			_Queue::default(),
			_Combos::new().with_mock(|mock| {
				mock.expect_peek_next().return_const(Skill {
					icon: Some(get_handle("combo skill")),
					..default()
				});
			}),
			_ComboTimeout(false),
		));
		let panel = app
			.world_mut()
			.spawn(QuickbarPanel {
				key: SlotKey::BottomHand(Side::Right),
				state: PanelState::Empty,
			})
			.id();

		app.update();

		let result = app.world().resource::<_Result>();
		assert_eq!(vec![(panel, Some(get_handle("combo skill")))], result.0);
	}

	fn setup_slots<const N: usize>(
		skills: [(SlotKey, SkillItemType, Skill); N],
	) -> (Slots, Assets<Item>, Assets<Skill>) {
		let mut slots = HashMap::new();
		let mut skill_assets = Assets::default();
		let mut item_assets = Assets::default();

		for (slot_key, item_type, skill) in skills {
			let skill = skill_assets.add(skill);
			let item = Item {
				item_type,
				skill: Some(skill),
				..default()
			};
			let item = item_assets.add(item);
			slots.insert(slot_key, Some(item));
		}

		(Slots(slots), item_assets, skill_assets)
	}

	#[test]
	fn peek_combo_with_correct_arguments() {
		let (slots, items, skills) = setup_slots([(
			SlotKey::BottomHand(Side::Left),
			SkillItemType::ForceEssence,
			Skill {
				icon: Some(get_handle("item skill")),
				..default()
			},
		)]);
		let mut app = setup(items, skills);
		app.world_mut().spawn((
			_Player,
			slots,
			_Queue::default(),
			_Combos::new().with_mock(|mock| {
				mock.expect_peek_next()
					.times(1)
					.with(
						eq(SlotKey::BottomHand(Side::Left)),
						eq(SkillItemType::ForceEssence),
					)
					.return_const(None);
			}),
			_ComboTimeout(false),
		));
		app.world_mut().spawn(QuickbarPanel {
			key: SlotKey::BottomHand(Side::Left),
			state: PanelState::Empty,
		});

		app.update();
	}

	#[test]
	fn return_item_skill_icon_when_no_skill_active_and_combo_timed_out() {
		let (slots, items, skills) = setup_slots([(
			SlotKey::BottomHand(Side::Right),
			SkillItemType::Pistol,
			Skill {
				icon: Some(get_handle("item skill")),
				..default()
			},
		)]);
		let mut app = setup(items, skills);
		app.world_mut().spawn((
			_Player,
			slots,
			_Queue::default(),
			_Combos::new().with_mock(|mock| {
				mock.expect_peek_next().return_const(Skill {
					icon: Some(get_handle("combo skill")),
					..default()
				});
			}),
			_ComboTimeout(true),
		));
		let panel = app
			.world_mut()
			.spawn(QuickbarPanel {
				key: SlotKey::BottomHand(Side::Right),
				state: PanelState::Empty,
			})
			.id();

		app.update();

		let result = app.world().resource::<_Result>();
		assert_eq!(vec![(panel, Some(get_handle("item skill")))], result.0);
	}

	#[test]
	fn return_item_skill_icon_when_no_skill_active_and_combo_empty_but_not_timed_out() {
		let (slots, items, skills) = setup_slots([(
			SlotKey::BottomHand(Side::Right),
			SkillItemType::Pistol,
			Skill {
				icon: Some(get_handle("item skill")),
				..default()
			},
		)]);
		let mut app = setup(items, skills);
		app.world_mut().spawn((
			_Player,
			slots,
			_Queue::default(),
			_Combos::new().with_mock(|mock| {
				mock.expect_peek_next().return_const(None);
			}),
			_ComboTimeout(false),
		));
		let panel = app
			.world_mut()
			.spawn(QuickbarPanel {
				key: SlotKey::BottomHand(Side::Right),
				state: PanelState::Empty,
			})
			.id();

		app.update();

		let result = app.world().resource::<_Result>();
		assert_eq!(vec![(panel, Some(get_handle("item skill")))], result.0);
	}

	#[test]
	fn return_active_skill_icon_when_skill_active() {
		let (slots, items, skills) = setup_slots([(
			SlotKey::BottomHand(Side::Right),
			SkillItemType::Pistol,
			Skill {
				icon: Some(get_handle("item skill")),
				..default()
			},
		)]);
		let mut app = setup(items, skills);
		app.world_mut().spawn((
			_Player,
			slots,
			_Queue(vec![QueuedSkill {
				skill: Skill {
					icon: Some(get_handle("active skill")),
					..default()
				},
				slot_key: SlotKey::BottomHand(Side::Left),
				mode: Activation::Waiting,
			}]),
			_Combos::new().with_mock(|mock| {
				mock.expect_peek_next().return_const(Skill {
					icon: Some(get_handle("combo skill")),
					..default()
				});
			}),
			_ComboTimeout(true),
		));
		let panel = app
			.world_mut()
			.spawn(QuickbarPanel {
				key: SlotKey::BottomHand(Side::Left),
				state: PanelState::Empty,
			})
			.id();

		app.update();

		let result = app.world().resource::<_Result>();
		assert_eq!(vec![(panel, Some(get_handle("active skill")))], result.0);
	}

	#[test]
	fn return_item_skill_icon_when_skill_active_for_other_slot() {
		let (slots, items, skills) = setup_slots([(
			SlotKey::BottomHand(Side::Right),
			SkillItemType::Pistol,
			Skill {
				icon: Some(get_handle("item skill")),
				..default()
			},
		)]);
		let mut app = setup(items, skills);
		app.world_mut().spawn((
			_Player,
			slots,
			_Queue(vec![QueuedSkill {
				skill: Skill {
					icon: Some(get_handle("active skill")),
					..default()
				},
				slot_key: SlotKey::BottomHand(Side::Left),
				mode: Activation::Waiting,
			}]),
			_Combos::new().with_mock(|mock| {
				mock.expect_peek_next().return_const(Skill {
					icon: Some(get_handle("combo skill")),
					..default()
				});
			}),
			_ComboTimeout(true),
		));
		let panel = app
			.world_mut()
			.spawn(QuickbarPanel {
				key: SlotKey::BottomHand(Side::Right),
				state: PanelState::Empty,
			})
			.id();

		app.update();

		let result = app.world().resource::<_Result>();
		assert_eq!(vec![(panel, Some(get_handle("item skill")))], result.0);
	}

	#[test]
	fn return_item_skill_icon_when_no_skill_active_and_no_combo_components_present() {
		let (slots, items, skills) = setup_slots([(
			SlotKey::BottomHand(Side::Right),
			SkillItemType::Pistol,
			Skill {
				icon: Some(get_handle("item skill")),
				..default()
			},
		)]);
		let mut app = setup(items, skills);
		app.world_mut().spawn((_Player, slots, _Queue::default()));
		let panel = app
			.world_mut()
			.spawn(QuickbarPanel {
				key: SlotKey::BottomHand(Side::Right),
				state: PanelState::Empty,
			})
			.id();

		app.update();

		let result = app.world().resource::<_Result>();
		assert_eq!(vec![(panel, Some(get_handle("item skill")))], result.0);
	}
}
