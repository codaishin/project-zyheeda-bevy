use crate::components::quickbar_panel::QuickbarPanel;
use bevy::prelude::*;
use common::{
	tools::{item_type::ItemType, slot_key::SlotKey},
	traits::{
		accessors::get::{GetField, GetFieldRef, Getter, GetterRef},
		handles_equipment::{IsTimedOut, ItemAsset, IterateQueue, PeekNext},
	},
};

#[allow(clippy::type_complexity)]
pub(crate) fn get_quickbar_icons<TPlayer, TSlots, TQueue, TCombos, TComboTimeout>(
	players: Query<(&TSlots, &TQueue, Option<&TCombos>, Option<&TComboTimeout>), With<TPlayer>>,
	panels: Query<(Entity, &mut QuickbarPanel)>,
	items: Res<Assets<TSlots::TItem>>,
	skills: Res<Assets<TCombos::TNext>>,
) -> Vec<(Entity, Option<Handle<Image>>)>
where
	TPlayer: Component,
	TSlots: Component + ItemAsset<TKey = SlotKey>,
	TComboTimeout: Component + IsTimedOut,
	TCombos: Component + PeekNext,
	TQueue: Component + IterateQueue,
	TSlots::TItem: Asset + Getter<ItemType> + GetterRef<Option<Handle<TCombos::TNext>>>,
	TCombos::TNext: Asset + GetterRef<Option<Handle<Image>>>,
	TQueue::TItem: GetterRef<Option<Handle<Image>>> + Getter<SlotKey>,
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

fn active_skill_icon<TQueue>(panel: &QuickbarPanel, queue: &TQueue) -> Option<Handle<Image>>
where
	TQueue: IterateQueue,
	TQueue::TItem: GetterRef<Option<Handle<Image>>> + Getter<SlotKey>,
{
	let active_skill = queue.iterate().next()?;
	let slot_key = SlotKey::get_field(active_skill);

	if slot_key != panel.key {
		return None;
	}

	let icon = Option::<Handle<Image>>::get_field_ref(active_skill);

	icon.clone()
}

fn combo_skill_icon<'a, TSlots, TCombos, TComboTimeout>(
	panel: &'a QuickbarPanel,
	items: &'a Assets<TSlots::TItem>,
	slots: &'a TSlots,
	combos: Option<&'a TCombos>,
	timed_out: Option<&'a TComboTimeout>,
) -> impl FnOnce() -> Option<Handle<Image>> + 'a
where
	TSlots: ItemAsset<TKey = SlotKey>,
	TSlots::TItem: Getter<ItemType>,
	TCombos: PeekNext,
	TCombos::TNext: GetterRef<Option<Handle<Image>>>,
	TComboTimeout: IsTimedOut,
{
	move || {
		if timed_out?.is_timed_out() {
			return None;
		}
		let item = slots
			.item_asset(&panel.key)
			.ok()
			.and_then(|handle| handle.as_ref())
			.and_then(|handle| items.get(handle))?;
		let next_combo = combos?.peek_next(&panel.key, &item.get())?;
		let icon = Option::<Handle<Image>>::get_field_ref(&next_combo);

		icon.clone()
	}
}

fn item_skill_icon<'a, TSlots, TSkill>(
	panel: &'a QuickbarPanel,
	items: &'a Assets<TSlots::TItem>,
	skills: &'a Assets<TSkill>,
	slots: &'a TSlots,
) -> impl FnOnce() -> Option<Handle<Image>> + 'a
where
	TSlots: ItemAsset<TKey = SlotKey>,
	TSlots::TItem: GetterRef<Option<Handle<TSkill>>>,
	TSkill: Asset + GetterRef<Option<Handle<Image>>>,
{
	|| {
		let Ok(slot) = slots.item_asset(&panel.key) else {
			return None;
		};

		slot.as_ref()
			.and_then(|item| items.get(item))
			.and_then(|item| Option::<Handle<TSkill>>::get_field_ref(item).as_ref())
			.and_then(|skill| skills.get(skill))
			.and_then(|skill| Option::<Handle<Image>>::get_field_ref(skill).clone())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{components::quickbar_panel::QuickbarPanel, tools::PanelState};
	use common::{
		test_tools::utils::SingleThreadedApp,
		tools::{
			item_type::ItemType,
			slot_key::{Side, SlotKey},
		},
		traits::{handles_equipment::KeyOutOfBounds, nested_mock::NestedMocks},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::collections::HashMap;
	use uuid::Uuid;

	#[derive(Component)]
	struct _Player;

	#[derive(Asset, TypePath, PartialEq, Debug, Clone)]
	pub struct _Skill(Option<Handle<Image>>);

	impl GetterRef<Option<Handle<Image>>> for _Skill {
		fn get(&self) -> &Option<Handle<Image>> {
			&self.0
		}
	}

	#[derive(Asset, TypePath, PartialEq, Debug)]
	struct _Item {
		skill: Option<Handle<_Skill>>,
		item_type: ItemType,
	}

	impl GetterRef<Option<Handle<_Skill>>> for _Item {
		fn get(&self) -> &Option<Handle<_Skill>> {
			&self.skill
		}
	}

	impl Getter<ItemType> for _Item {
		fn get(&self) -> ItemType {
			self.item_type
		}
	}

	struct _QueuedSkill {
		icon: Option<Handle<Image>>,
		slot_key: SlotKey,
	}

	impl GetterRef<Option<Handle<Image>>> for _QueuedSkill {
		fn get(&self) -> &Option<Handle<Image>> {
			&self.icon
		}
	}

	impl Getter<SlotKey> for _QueuedSkill {
		fn get(&self) -> SlotKey {
			self.slot_key
		}
	}

	#[derive(Component, Default)]
	struct _Queue(Vec<_QueuedSkill>);

	impl IterateQueue for _Queue {
		type TItem = _QueuedSkill;

		fn iterate(&self) -> impl Iterator<Item = &Self::TItem> {
			self.0.iter()
		}
	}

	#[derive(Component, NestedMocks)]
	struct _Combos {
		mock: Mock_Combos,
	}

	#[automock]
	impl PeekNext for _Combos {
		type TNext = _Skill;

		fn peek_next(&self, trigger: &SlotKey, item_type: &ItemType) -> Option<_Skill> {
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

	#[derive(Component)]
	struct _Slots(HashMap<SlotKey, Option<Handle<_Item>>>);

	impl ItemAsset for _Slots {
		type TKey = SlotKey;
		type TItem = _Item;

		fn item_asset(
			&self,
			key: &Self::TKey,
		) -> Result<&Option<Handle<Self::TItem>>, KeyOutOfBounds> {
			let Some(item) = self.0.get(key) else {
				return Err(KeyOutOfBounds);
			};

			Ok(item)
		}
	}

	#[derive(Resource, Default)]
	struct _Result(Vec<(Entity, Option<Handle<Image>>)>);

	fn store_result(result: In<Vec<(Entity, Option<Handle<Image>>)>>, mut commands: Commands) {
		commands.insert_resource(_Result(result.0));
	}

	fn setup(items: Assets<_Item>, skills: Assets<_Skill>) -> App {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(items);
		app.insert_resource(skills);
		app.init_resource::<_Result>();
		app.add_systems(
			Update,
			get_quickbar_icons::<_Player, _Slots, _Queue, _Combos, _ComboTimeout>
				.pipe(store_result),
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
			ItemType::Pistol,
			_Skill(Some(get_handle("item skill"))),
		)]);
		let mut app = setup(items, skills);
		app.world_mut().spawn((
			_Player,
			slots,
			_Queue::default(),
			_Combos::new().with_mock(|mock| {
				mock.expect_peek_next()
					.return_const(_Skill(Some(get_handle("combo skill"))));
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
		skills: [(SlotKey, ItemType, _Skill); N],
	) -> (_Slots, Assets<_Item>, Assets<_Skill>) {
		let mut slots = HashMap::new();
		let mut skill_assets = Assets::default();
		let mut item_assets = Assets::default();

		for (slot_key, item_type, skill) in skills {
			let skill = skill_assets.add(skill);
			let item = _Item {
				item_type,
				skill: Some(skill),
			};
			let item = item_assets.add(item);
			slots.insert(slot_key, Some(item));
		}

		(_Slots(slots), item_assets, skill_assets)
	}

	#[test]
	fn peek_combo_with_correct_arguments() {
		let (slots, items, skills) = setup_slots([(
			SlotKey::BottomHand(Side::Left),
			ItemType::ForceEssence,
			_Skill(Some(get_handle("item skill"))),
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
						eq(ItemType::ForceEssence),
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
			ItemType::Pistol,
			_Skill(Some(get_handle("item skill"))),
		)]);
		let mut app = setup(items, skills);
		app.world_mut().spawn((
			_Player,
			slots,
			_Queue::default(),
			_Combos::new().with_mock(|mock| {
				mock.expect_peek_next()
					.return_const(_Skill(Some(get_handle("combo skill"))));
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
			ItemType::Pistol,
			_Skill(Some(get_handle("item skill"))),
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
			ItemType::Pistol,
			_Skill(Some(get_handle("item skill"))),
		)]);
		let mut app = setup(items, skills);
		app.world_mut().spawn((
			_Player,
			slots,
			_Queue(vec![_QueuedSkill {
				icon: Some(get_handle("active skill")),
				slot_key: SlotKey::BottomHand(Side::Left),
			}]),
			_Combos::new().with_mock(|mock| {
				mock.expect_peek_next()
					.return_const(_Skill(Some(get_handle("combo skill"))));
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
			ItemType::Pistol,
			_Skill(Some(get_handle("item skill"))),
		)]);
		let mut app = setup(items, skills);
		app.world_mut().spawn((
			_Player,
			slots,
			_Queue(vec![_QueuedSkill {
				icon: Some(get_handle("active skill")),
				slot_key: SlotKey::BottomHand(Side::Left),
			}]),
			_Combos::new().with_mock(|mock| {
				mock.expect_peek_next()
					.return_const(_Skill(Some(get_handle("combo skill"))));
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
			ItemType::Pistol,
			_Skill(Some(get_handle("item skill"))),
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
